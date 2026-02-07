use std::{convert::Infallible, net::SocketAddr, time::Duration};

use anyhow::Context;
use async_stream::stream;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, sse::{Event, KeepAlive, Sse}},
    routing::{get, post},
};
use serde_json::json;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

use crate::{
    executor::scene_plan_to_ops,
    planner::{PlanningContext, build_scene_plan},
    policy::RiskTier,
    protocol::{
        DonePayload, ErrorPayload, PingPayload, RunPromptRequest, RunPromptResponse,
        SessionCreateResponse, SsePayload, ToastLevel, ToastPayload, UiEvent, UiEventAck, UiOp,
        UiOpsPayload, UiPatchOp, UiProps, UiEventRequest,
    },
    session::{OutboundMessage, SessionContext, SessionManager},
};

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub sessions: SessionManager,
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8099,
        }
    }
}

pub fn app_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/api/3d/session", post(create_session))
        .route("/api/3d/stream/{session_id}", get(stream_session))
        .route("/api/3d/event/{session_id}", post(post_ui_event))
        .route("/api/3d/run/{session_id}", post(run_prompt))
        .with_state(state)
        .layer(cors)
}

pub async fn run_server(config: ServerConfig) -> anyhow::Result<()> {
    let state = AppState::default();
    let app = app_router(state);
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .with_context(|| "invalid host/port for adk-3d-ui server")?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("adk-3d-ui listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> impl IntoResponse {
    Html(include_str!("../ui/index.html"))
}

async fn health() -> impl IntoResponse {
    Json(json!({"status":"ok","service":"adk-3d-ui"}))
}

async fn create_session(State(state): State<AppState>) -> impl IntoResponse {
    let session_id = state.sessions.create_session().await;
    Json(SessionCreateResponse { session_id })
}

async fn stream_session(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    state.sessions.ensure_session(&session_id).await;
    let mut rx = state
        .sessions
        .subscribe(&session_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    let _ = state
        .sessions
        .publish(&session_id, SsePayload::Ping(PingPayload::now()))
        .await;

    let stream = stream! {
        loop {
            match rx.recv().await {
                Ok(OutboundMessage { event, data }) => {
                    yield Ok(Event::default().event(event).data(data));
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    let warn = json!({"warning":"client lagged","skipped": skipped});
                    yield Ok(Event::default().event("log").data(warn.to_string()));
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)).text("keepalive")))
}

async fn post_ui_event(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<UiEventRequest>,
) -> Result<Json<UiEventAck>, (StatusCode, Json<UiEventAck>)> {
    state.sessions.ensure_session(&session_id).await;

    state
        .sessions
        .record_event(&session_id, request.clone())
        .await
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(UiEventAck {
                    ok: false,
                    server_seq: None,
                    error: Some("session_not_found".to_string()),
                }),
            )
        })?;

    match &request.event {
        UiEvent::Command { text } => {
            let _ = state
                .sessions
                .set_last_command(&session_id, text.to_string())
                .await;
            publish_plan_for_prompt(&state, &session_id, text, Some(request.seq)).await;
        }
        UiEvent::Select { id } => {
            let _ = state
                .sessions
                .set_selected_id(&session_id, Some(id.to_string()))
                .await;
            publish_focus_patch_for_selection(&state, &session_id, id, Some(request.seq)).await;
        }
        UiEvent::ApproveAction {
            action_id,
            approved,
        } => {
            let message = if *approved {
                format!("Approved action `{action_id}`.")
            } else {
                format!("Rejected action `{action_id}`.")
            };
            let _ = state
                .sessions
                .publish(
                    &session_id,
                    SsePayload::Toast(ToastPayload {
                        level: if *approved {
                            ToastLevel::Success
                        } else {
                            ToastLevel::Info
                        },
                        message,
                    }),
                )
                .await;
        }
    }

    let server_seq = state.sessions.last_server_seq(&session_id).await;
    Ok(Json(UiEventAck {
        ok: true,
        server_seq,
        error: None,
    }))
}

async fn run_prompt(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<RunPromptRequest>,
) -> Result<Json<RunPromptResponse>, (StatusCode, Json<RunPromptResponse>)> {
    if request.prompt.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(RunPromptResponse {
                accepted: false,
                message: "prompt cannot be empty".to_string(),
            }),
        ));
    }

    state.sessions.ensure_session(&session_id).await;
    publish_plan_for_prompt(&state, &session_id, &request.prompt, None).await;

    Ok(Json(RunPromptResponse {
        accepted: true,
        message: "scene plan emitted".to_string(),
    }))
}

async fn publish_plan_for_prompt(
    state: &AppState,
    session_id: &str,
    prompt: &str,
    reply_to: Option<u64>,
) {
    let _ = state
        .sessions
        .set_last_prompt(session_id, prompt.to_string())
        .await;

    let context = state
        .sessions
        .get_context(session_id)
        .await
        .unwrap_or_default();
    let planning_context = planning_context_from_session(&context);
    let plan = build_scene_plan(prompt, &planning_context);

    let _ = state
        .sessions
        .update_plan_state(
            session_id,
            format!("{:?}", plan.intent.domain).to_lowercase(),
            plan.nodes.iter().map(|node| node.id.clone()).collect(),
        )
        .await;

    let _ = state
        .sessions
        .publish(
            session_id,
            SsePayload::Toast(ToastPayload {
                level: ToastLevel::Info,
                message: "Planning 3D scene from prompt...".to_string(),
            }),
        )
        .await;

    if let Some(action) = &plan.action {
        let message = match action.risk {
            RiskTier::Dangerous => {
                "Dangerous action detected. Approval will be required before execution."
            }
            RiskTier::Controlled => "Controlled action detected in prompt intent.",
            RiskTier::Safe => "Safe mode actions only.",
        };
        let _ = state
            .sessions
            .publish(
                session_id,
                SsePayload::Toast(ToastPayload {
                    level: if matches!(action.risk, RiskTier::Dangerous) {
                        ToastLevel::Warning
                    } else {
                        ToastLevel::Info
                    },
                    message: message.to_string(),
                }),
            )
            .await;
    }

    let ops = scene_plan_to_ops(&plan, reply_to);

    if state
        .sessions
        .publish(session_id, SsePayload::UiOps(ops))
        .await
        .is_none()
    {
        error!(session_id, "failed to publish ui_ops");
        let _ = state
            .sessions
            .publish(
                session_id,
                SsePayload::Error(ErrorPayload {
                    code: "publish_failed".to_string(),
                    message: "failed to stream ui ops".to_string(),
                }),
            )
            .await;
        return;
    }

    let _ = state
        .sessions
        .publish(
            session_id,
            SsePayload::Done(DonePayload {
                status: "completed".to_string(),
            }),
        )
        .await;

    if !plan.nodes.is_empty() {
        spawn_live_status_patch_loop(
            state.clone(),
            session_id.to_string(),
            plan.nodes.iter().map(|n| n.id.clone()).collect(),
        );
    }
}

fn planning_context_from_session(context: &SessionContext) -> PlanningContext {
    PlanningContext {
        last_prompt: context.last_prompt.clone(),
        last_command: context.last_command.clone(),
        selected_id: context.selected_id.clone(),
    }
}

async fn publish_focus_patch_for_selection(
    state: &AppState,
    session_id: &str,
    selected_id: &str,
    reply_to: Option<u64>,
) {
    let context = state
        .sessions
        .get_context(session_id)
        .await
        .unwrap_or_default();

    let mut ops = Vec::new();
    for node_id in &context.last_node_ids {
        let mut props = UiProps::new();
        props.insert("selected".to_string(), json!(node_id == selected_id));
        ops.push(UiOp::Patch(UiPatchOp {
            id: node_id.to_string(),
            props,
        }));
    }

    let mut panel_props = UiProps::new();
    panel_props.insert(
        "title".to_string(),
        json!(format!("Service Workbench: {selected_id}")),
    );
    panel_props.insert(
        "subtitle".to_string(),
        json!(format!(
            "Selected node: {selected_id}. Investigate logs, traces, and deployment diffs."
        )),
    );
    ops.push(UiOp::Patch(UiPatchOp {
        id: "workbench-panel".to_string(),
        props: panel_props.clone(),
    }));

    if !context.last_node_ids.is_empty() {
        let create_if_missing = serde_json::json!({
            "op": "create",
            "id": "workbench-panel",
            "kind": "panel3d",
            "parent": "root",
            "props": panel_props,
        });
        if let Ok(create_op) = serde_json::from_value::<UiOp>(create_if_missing) {
            ops.insert(0, create_op);
        }
    }

    let _ = state
        .sessions
        .publish(
            session_id,
            SsePayload::UiOps(UiOpsPayload { reply_to, ops }),
        )
        .await;
}

fn spawn_live_status_patch_loop(state: AppState, session_id: String, node_ids: Vec<String>) {
    tokio::spawn(async move {
        let phases = [
            ["healthy", "warning", "degraded"],
            ["warning", "degraded", "critical"],
            ["degraded", "healthy", "warning"],
        ];

        for phase in phases {
            tokio::time::sleep(Duration::from_millis(900)).await;
            let mut ops = Vec::new();
            for (idx, node_id) in node_ids.iter().enumerate() {
                let mut props = UiProps::new();
                let status = phase[idx % phase.len()];
                props.insert("status".to_string(), json!(status));
                ops.push(UiOp::Patch(UiPatchOp {
                    id: node_id.clone(),
                    props,
                }));
            }
            let _ = state
                .sessions
                .publish(
                    &session_id,
                    SsePayload::UiOps(UiOpsPayload {
                        reply_to: None,
                        ops,
                    }),
                )
                .await;
        }

        let _ = state
            .sessions
            .publish(
                &session_id,
                SsePayload::Toast(ToastPayload {
                    level: ToastLevel::Info,
                    message: "Live status patch loop completed.".to_string(),
                }),
            )
            .await;
    });
}
