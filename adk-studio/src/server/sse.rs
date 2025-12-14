use crate::compiler::compile_agent;
use crate::server::state::AppState;
use adk_core::Content;
use adk_runner::{Runner, RunnerConfig};
use adk_session::{CreateRequest, GetRequest, InMemorySessionService, SessionService};
use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, Sse},
};
use futures::{Stream, StreamExt};
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, OnceLock};

fn session_service() -> &'static Arc<InMemorySessionService> {
    static INSTANCE: OnceLock<Arc<InMemorySessionService>> = OnceLock::new();
    INSTANCE.get_or_init(|| Arc::new(InMemorySessionService::new()))
}

#[derive(Deserialize)]
pub struct StreamQuery {
    input: String,
    #[serde(default)]
    api_key: Option<String>,
}

pub async fn stream_handler(
    Path(id): Path<String>,
    Query(query): Query<StreamQuery>,
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let api_key = query.api_key
        .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
        .unwrap_or_default();

    let stream = async_stream::stream! {
        let project_id: uuid::Uuid = match id.parse() {
            Ok(id) => id,
            Err(e) => {
                yield Ok(Event::default().event("error").data(e.to_string()));
                return;
            }
        };

        let storage = state.storage.read().await;
        let project = match storage.get(project_id).await {
            Ok(p) => p,
            Err(e) => {
                yield Ok(Event::default().event("error").data(e.to_string()));
                return;
            }
        };

        let (agent_name, agent_schema) = match project.agents.iter().next() {
            Some(a) => a,
            None => {
                yield Ok(Event::default().event("error").data("No agents"));
                return;
            }
        };

        let agent = match compile_agent(agent_name, agent_schema, &api_key) {
            Ok(a) => a,
            Err(e) => {
                yield Ok(Event::default().event("error").data(e.to_string()));
                return;
            }
        };
        let agent_name = agent_name.to_string();
        drop(storage);

        yield Ok(Event::default().event("start").data(&agent_name));

        let svc = session_service().clone();
        let session_id = project_id.to_string();

        let session = match svc.get(GetRequest {
            app_name: "studio".into(),
            user_id: "user".into(),
            session_id: session_id.clone(),
            num_recent_events: None,
            after: None,
        }).await {
            Ok(s) => s,
            Err(_) => match svc.create(CreateRequest {
                app_name: "studio".into(),
                user_id: "user".into(),
                session_id: Some(session_id),
                state: HashMap::new(),
            }).await {
                Ok(s) => s,
                Err(e) => {
                    yield Ok(Event::default().event("error").data(e.to_string()));
                    return;
                }
            }
        };

        let runner = match Runner::new(RunnerConfig {
            app_name: "studio".into(),
            agent,
            session_service: svc,
            artifact_service: None,
            memory_service: None,
        }) {
            Ok(r) => r,
            Err(e) => {
                yield Ok(Event::default().event("error").data(e.to_string()));
                return;
            }
        };

        let content = Content::new("user").with_text(&query.input);
        let mut run_stream = match runner.run("user".into(), session.id().to_string(), content).await {
            Ok(s) => s,
            Err(e) => {
                yield Ok(Event::default().event("error").data(e.to_string()));
                return;
            }
        };

        while let Some(result) = run_stream.next().await {
            if let Ok(event) = result {
                if let Some(c) = event.content() {
                    for part in &c.parts {
                        if let Some(text) = part.text() {
                            yield Ok(Event::default().event("chunk").data(text));
                        }
                    }
                }
            }
        }

        yield Ok(Event::default().event("end").data(""));
    };

    Sse::new(stream)
}
