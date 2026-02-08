//! Bridge points for connecting shell app lifecycle to ADK-Rust agent runtimes.

use std::{collections::HashMap, sync::Arc};

use adk_agent::LlmAgentBuilder;
use adk_core::{Agent, Content, Llm};
use adk_model::GeminiModel;
use adk_runner::{Runner, RunnerConfig};
use adk_session::{CreateRequest, InMemorySessionService, SessionService};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use super::manifest::AppManifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppExecutionInput {
    pub app: AppManifest,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppExecutionOutput {
    pub app_id: String,
    pub accepted: bool,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn resolve_model_name(input: &AppExecutionInput) -> String {
    input
        .app
        .runtime
        .default_model
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .or_else(|| {
            std::env::var("ADK_SPATIAL_OS_MODEL")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "gemini-2.5-flash".to_string())
}

fn resolve_api_key() -> Option<String> {
    std::env::var("GOOGLE_API_KEY")
        .ok()
        .or_else(|| std::env::var("GEMINI_API_KEY").ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn resolve_vertex_project_location() -> Option<(String, String)> {
    let project = std::env::var("GOOGLE_PROJECT_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())?;
    let location = std::env::var("GOOGLE_CLOUD_LOCATION")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())?;
    Some((project, location))
}

fn provider_hint(input: &AppExecutionInput) -> Option<String> {
    input
        .app
        .runtime
        .provider
        .as_ref()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("ADK_SPATIAL_OS_MODEL_PROVIDER")
                .ok()
                .map(|value| value.trim().to_lowercase())
                .filter(|value| !value.is_empty())
        })
}

fn select_model(input: &AppExecutionInput) -> Result<(Arc<dyn Llm>, String), String> {
    let model_name = resolve_model_name(input);
    let key = resolve_api_key();
    let vertex = resolve_vertex_project_location();
    let provider = provider_hint(input).unwrap_or_else(|| "auto".to_string());

    let prefer_vertex =
        provider.contains("vertex") || (provider == "auto" && key.is_none() && vertex.is_some());
    if prefer_vertex {
        if let Some((project, location)) = vertex.as_ref() {
            match GeminiModel::new_google_cloud_adc(project, location, model_name.clone()) {
                Ok(model) => {
                    return Ok((
                        Arc::new(model) as Arc<dyn Llm>,
                        format!("vertex_adc:{}@{}", model_name, location),
                    ));
                }
                Err(error) => {
                    if key.is_none() {
                        return Err(format!(
                            "failed to initialize Vertex ADC model for `{}` in `{}`: {error}",
                            project, location
                        ));
                    }
                }
            }
        } else if key.is_none() {
            return Err(
                "provider `vertex_adc` requires GOOGLE_PROJECT_ID and GOOGLE_CLOUD_LOCATION"
                    .to_string(),
            );
        }
    }

    if let Some(api_key) = key {
        return GeminiModel::new(api_key, model_name.clone())
            .map(|model| (Arc::new(model) as Arc<dyn Llm>, format!("ai_studio:{model_name}")))
            .map_err(|error| format!("failed to initialize AI Studio model: {error}"));
    }

    if let Some((project, location)) = vertex {
        return GeminiModel::new_google_cloud_adc(&project, &location, model_name.clone())
            .map(|model| {
                (Arc::new(model) as Arc<dyn Llm>, format!("vertex_adc:{}@{}", model_name, location))
            })
            .map_err(|error| format!("failed to initialize Vertex ADC model: {error}"));
    }

    Err(
        "no model credentials found; set GOOGLE_API_KEY/GEMINI_API_KEY or GOOGLE_PROJECT_ID + GOOGLE_CLOUD_LOCATION"
            .to_string(),
    )
}

fn summary_limit() -> usize {
    std::env::var("ADK_SPATIAL_OS_SUMMARY_MAX_CHARS")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 160)
        .unwrap_or(1600)
}

fn truncate_summary(text: &str, max_chars: usize) -> String {
    let mut truncated = String::new();
    for ch in text.chars() {
        if truncated.chars().count() >= max_chars {
            break;
        }
        truncated.push(ch);
    }
    if truncated.chars().count() < text.chars().count() {
        truncated.push_str("…");
    }
    truncated
}

pub async fn run_app_agent(input: AppExecutionInput) -> AppExecutionOutput {
    let app_id = input.app.id.clone();
    let (model, model_label) = match select_model(&input) {
        Ok(model) => model,
        Err(error) => {
            return AppExecutionOutput {
                app_id,
                accepted: false,
                summary: format!("Execution failed: {error}"),
                model: None,
                error: Some(error),
            };
        }
    };

    let instruction = input
        .app
        .runtime
        .instruction
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| format!("You are {}. {}", input.app.name, input.app.description));

    let agent = match LlmAgentBuilder::new(input.app.id.clone())
        .description(input.app.description.clone())
        .instruction(instruction)
        .model(model)
        .build()
    {
        Ok(agent) => Arc::new(agent) as Arc<dyn Agent>,
        Err(error) => {
            let message = format!("failed to build runtime agent: {error}");
            return AppExecutionOutput {
                app_id,
                accepted: false,
                summary: format!("Execution failed: {message}"),
                model: Some(model_label),
                error: Some(message),
            };
        }
    };

    let user_id = "spatial-os-user".to_string();
    let session_id = format!("{}-{}", input.app.id, uuid::Uuid::new_v4());
    let session_service = Arc::new(InMemorySessionService::new());
    if let Err(error) = session_service
        .create(CreateRequest {
            app_name: input.app.id.clone(),
            user_id: user_id.clone(),
            session_id: Some(session_id.clone()),
            state: HashMap::new(),
        })
        .await
    {
        let message = format!("failed to create app session: {error}");
        return AppExecutionOutput {
            app_id,
            accepted: false,
            summary: format!("Execution failed: {message}"),
            model: Some(model_label),
            error: Some(message),
        };
    }

    let runner = match Runner::new(RunnerConfig {
        app_name: input.app.id.clone(),
        agent,
        session_service,
        artifact_service: None,
        memory_service: None,
        plugin_manager: None,
        run_config: None,
        compaction_config: None,
    }) {
        Ok(runner) => runner,
        Err(error) => {
            let message = format!("failed to initialize runner: {error}");
            return AppExecutionOutput {
                app_id,
                accepted: false,
                summary: format!("Execution failed: {message}"),
                model: Some(model_label),
                error: Some(message),
            };
        }
    };

    let mut stream = match runner
        .run(user_id, session_id, Content::new("user").with_text(input.prompt.clone()))
        .await
    {
        Ok(stream) => stream,
        Err(error) => {
            let message = format!("runner invocation failed: {error}");
            return AppExecutionOutput {
                app_id,
                accepted: false,
                summary: format!("Execution failed: {message}"),
                model: Some(model_label),
                error: Some(message),
            };
        }
    };

    let mut fragments: Vec<String> = Vec::new();
    while let Some(next) = stream.next().await {
        match next {
            Ok(event) => {
                if let Some(content) = event.llm_response.content {
                    for part in content.parts {
                        if let Some(text) = part.text() {
                            let chunk = text.trim();
                            if !chunk.is_empty() {
                                fragments.push(chunk.to_string());
                            }
                        }
                    }
                }
            }
            Err(error) => {
                let message = format!("execution stream failed: {error}");
                return AppExecutionOutput {
                    app_id,
                    accepted: false,
                    summary: format!("Execution failed: {message}"),
                    model: Some(model_label),
                    error: Some(message),
                };
            }
        }
    }

    let summary = if fragments.is_empty() {
        "Agent run completed without text output.".to_string()
    } else {
        let joined = fragments.join("\n");
        truncate_summary(joined.trim(), summary_limit())
    };

    AppExecutionOutput { app_id, accepted: true, summary, model: Some(model_label), error: None }
}
