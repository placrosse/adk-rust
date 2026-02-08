use std::sync::Arc;

use async_trait::async_trait;

use crate::safety::risk::{RiskTier, classify_prompt_risk};

use super::{
    bridge::{AppExecutionInput, run_app_agent},
    manifest::{AppManifest, default_manifests},
};

#[derive(Debug, Clone)]
pub struct IntentRoute {
    pub selected_apps: Vec<String>,
    pub risk: RiskTier,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub struct CommandDispatch {
    pub accepted: bool,
    pub summary: String,
}

#[async_trait]
pub trait AgentAppHost: Send + Sync {
    async fn list_apps(&self) -> Vec<AppManifest>;
    async fn route_prompt(&self, prompt: &str) -> IntentRoute;
    async fn execute_command(&self, app_id: &str, command: &str) -> CommandDispatch;
}

#[derive(Debug, Clone)]
pub struct InMemoryAgentHost {
    apps: Arc<Vec<AppManifest>>,
}

impl Default for InMemoryAgentHost {
    fn default() -> Self {
        Self {
            apps: Arc::new(default_manifests()),
        }
    }
}

#[async_trait]
impl AgentAppHost for InMemoryAgentHost {
    async fn list_apps(&self) -> Vec<AppManifest> {
        self.apps.as_ref().clone()
    }

    async fn route_prompt(&self, prompt: &str) -> IntentRoute {
        let p = prompt.to_lowercase();
        let risk = classify_prompt_risk(prompt);
        let tokens: Vec<String> = p
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect();

        let mut scored: Vec<(String, i32)> = self
            .apps
            .iter()
            .map(|app| {
                let mut score = 0;
                let haystack = format!(
                    "{} {} {}",
                    app.name.to_lowercase(),
                    app.description.to_lowercase(),
                    app.capabilities.join(" ").to_lowercase(),
                );
                for token in &tokens {
                    if haystack.contains(token) {
                        score += 3;
                    }
                    match token.as_str() {
                        "incident" | "service" | "rollback" | "restart" | "ops" if app.id == "ops-center" => score += 5,
                        "mail" | "email" | "inbox" if app.id == "mail-agent" => score += 5,
                        "calendar" | "meeting" | "schedule" if app.id == "calendar-agent" => score += 5,
                        _ => {}
                    }
                }
                (app.id.clone(), score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let mut selected: Vec<String> = scored
            .iter()
            .filter(|(_, score)| *score > 0)
            .take(2)
            .map(|(app_id, _)| app_id.clone())
            .collect();

        if selected.is_empty() {
            selected.push("ops-center".to_string());
        }

        let rationale = {
            let top = scored
                .into_iter()
                .filter(|(_, score)| *score > 0)
                .take(3)
                .map(|(app, score)| format!("{app}:{score}"))
                .collect::<Vec<_>>();
            if top.is_empty() {
                "fallback route selected: ops-center".to_string()
            } else {
                format!("capability score route selected ({})", top.join(", "))
            }
        };

        IntentRoute {
            selected_apps: selected,
            risk,
            rationale,
        }
    }

    async fn execute_command(&self, app_id: &str, command: &str) -> CommandDispatch {
        let app_exists = self.apps.iter().any(|app| app.id == app_id);
        if !app_exists {
            return CommandDispatch {
                accepted: false,
                summary: format!("Unknown app: {app_id}"),
            };
        }

        let output = run_app_agent(AppExecutionInput {
            app_id: app_id.to_string(),
            prompt: command.to_string(),
        })
        .await;
        CommandDispatch {
            accepted: true,
            summary: output.summary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentAppHost, InMemoryAgentHost};

    #[tokio::test]
    async fn route_prompt_prefers_capability_scored_apps() {
        let host = InMemoryAgentHost::default();
        let route = host
            .route_prompt("schedule a meeting and fix calendar conflicts")
            .await;
        assert!(
            route.selected_apps.iter().any(|app| app == "calendar-agent"),
            "expected calendar-agent in selected apps"
        );
        assert!(
            route.rationale.contains("capability score route selected")
                || route.rationale.contains("fallback route selected"),
            "unexpected rationale format: {}",
            route.rationale
        );
    }

    #[tokio::test]
    async fn execute_command_rejects_unknown_app() {
        let host = InMemoryAgentHost::default();
        let dispatch = host.execute_command("unknown-app", "do something").await;
        assert!(!dispatch.accepted);
        assert!(dispatch.summary.contains("Unknown app"));
    }
}
