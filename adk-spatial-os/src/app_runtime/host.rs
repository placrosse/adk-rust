use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::safety::risk::{RiskTier, classify_prompt_risk, max_risk, risk_rank};

use super::{
    bridge::{AppExecutionInput, run_app_agent},
    handoff::HandoffPolicyDecision,
    manifest::{AppManifest, AppRuntimeConfig, default_manifests},
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

#[derive(Debug, Clone)]
pub struct AppRegistration {
    pub created: bool,
    pub app_id: String,
}

#[async_trait]
pub trait AgentAppHost: Send + Sync {
    async fn list_apps(&self) -> Vec<AppManifest>;
    async fn route_prompt(&self, prompt: &str) -> IntentRoute;
    async fn execute_command(&self, app_id: &str, command: &str) -> CommandDispatch;
    async fn evaluate_handoff_policy(&self, from_app: &str, to_app: &str) -> HandoffPolicyDecision;
    async fn upsert_app(&self, manifest: AppManifest) -> AppRegistration;
}

#[derive(Debug, Clone)]
pub struct InMemoryAgentHost {
    apps: Arc<RwLock<Vec<AppManifest>>>,
}

impl Default for InMemoryAgentHost {
    fn default() -> Self {
        Self { apps: Arc::new(RwLock::new(default_manifests())) }
    }
}

#[derive(Debug, Deserialize)]
struct RouteDecision {
    #[serde(default)]
    selected_apps: Vec<String>,
    #[serde(default)]
    risk: Option<String>,
    #[serde(default)]
    rationale: Option<String>,
}

fn route_max_apps() -> usize {
    std::env::var("ADK_SPATIAL_OS_ROUTE_MAX_APPS")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(2)
}

fn tokenize(value: &str) -> Vec<String> {
    value
        .to_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn app_search_blob(app: &AppManifest) -> String {
    let mut fields = vec![
        app.id.clone(),
        app.name.clone(),
        app.description.clone(),
        app.capabilities.join(" "),
        app.permissions.join(" "),
        app.starter_prompts.join(" "),
        app.runtime.mode.clone(),
        app.runtime.workflow_type.clone().unwrap_or_default(),
        app.runtime.instruction.clone().unwrap_or_default(),
    ];
    fields.retain(|value| !value.trim().is_empty());
    fields.join(" ").to_lowercase()
}

fn parse_risk_tier(value: Option<&str>) -> Option<RiskTier> {
    match value?.trim().to_lowercase().as_str() {
        "safe" => Some(RiskTier::Safe),
        "controlled" => Some(RiskTier::Controlled),
        "dangerous" => Some(RiskTier::Dangerous),
        _ => None,
    }
}

fn max_risk_for_selected(selected_apps: &[String], apps: &[AppManifest]) -> RiskTier {
    selected_apps
        .iter()
        .filter_map(|app_id| apps.iter().find(|app| app.id == *app_id))
        .map(|app| app.default_risk)
        .max_by_key(|risk| risk_rank(*risk))
        .unwrap_or(RiskTier::Safe)
}

fn normalize_route_selection(
    candidate_ids: &[String],
    apps: &[AppManifest],
    max_apps: usize,
) -> Vec<String> {
    let mut selected = Vec::new();
    for app_id in candidate_ids {
        if selected.len() >= max_apps {
            break;
        }
        if !apps.iter().any(|app| app.id == *app_id) {
            continue;
        }
        if selected.iter().any(|existing| existing == app_id) {
            continue;
        }
        selected.push(app_id.clone());
    }

    if selected.is_empty() {
        if let Some(first) = apps.first() {
            selected.push(first.id.clone());
        }
    }

    selected
}

fn extract_json_object(input: &str) -> Option<&str> {
    let trimmed = input.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed);
    }
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(&trimmed[start..=end])
}

async fn route_with_llm(prompt: &str, apps: &[AppManifest]) -> Option<IntentRoute> {
    if apps.is_empty() {
        return None;
    }

    let max_apps = route_max_apps();
    let app_catalog = apps
        .iter()
        .map(|app| {
            serde_json::json!({
                "id": app.id,
                "name": app.name,
                "description": app.description,
                "capabilities": app.capabilities,
                "permissions": app.permissions,
                "default_risk": app.default_risk,
            })
        })
        .collect::<Vec<_>>();

    let router_instruction = r#"
You route user requests to the best ADK apps.
Return JSON only, no markdown:
{
  "selected_apps": ["app-id-1", "app-id-2"],
  "risk": "safe|controlled|dangerous",
  "rationale": "one short sentence"
}
Rules:
- selected_apps must contain 1 or 2 ids from AVAILABLE_APPS.
- Choose smallest set of apps needed.
- risk reflects expected operational impact.
"#;

    let router_app = AppManifest {
        id: "master-router".to_string(),
        name: "Master Router".to_string(),
        version: "1.0.0".to_string(),
        description: "Manifest-aware route selection".to_string(),
        capabilities: vec!["routing".to_string()],
        permissions: vec![],
        handoff_allowlist: vec![],
        default_risk: RiskTier::Safe,
        starter_prompts: vec![],
        runtime: AppRuntimeConfig {
            default_model: std::env::var("ADK_SPATIAL_OS_ROUTER_MODEL")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            instruction: Some(router_instruction.trim().to_string()),
            ..AppRuntimeConfig::default()
        },
    };

    let router_prompt = format!(
        "AVAILABLE_APPS:\n{}\n\nUSER_PROMPT:\n{}\n\nReturn JSON only.",
        serde_json::to_string_pretty(&app_catalog).ok()?,
        prompt
    );

    let output = run_app_agent(AppExecutionInput { app: router_app, prompt: router_prompt }).await;
    if !output.accepted {
        return None;
    }

    let raw_json = extract_json_object(&output.summary)?;
    let parsed = serde_json::from_str::<RouteDecision>(raw_json).ok()?;
    let selected_apps = normalize_route_selection(&parsed.selected_apps, apps, max_apps);
    if selected_apps.is_empty() {
        return None;
    }

    let selected_risk = parse_risk_tier(parsed.risk.as_deref())
        .unwrap_or_else(|| max_risk_for_selected(&selected_apps, apps));
    let risk = max_risk(selected_risk, classify_prompt_risk(prompt));
    let rationale = parsed
        .rationale
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "LLM route selected based on manifest capabilities".to_string());

    Some(IntentRoute { selected_apps, risk, rationale })
}

fn route_with_manifest_similarity(prompt: &str, apps: &[AppManifest]) -> IntentRoute {
    let max_apps = route_max_apps();
    let tokens = tokenize(prompt);
    let mut scored = apps
        .iter()
        .map(|app| {
            let haystack = app_search_blob(app);
            let score = tokens
                .iter()
                .map(|token| {
                    if haystack.contains(token) {
                        if app.capabilities.iter().any(|cap| cap.to_lowercase().contains(token)) {
                            3
                        } else {
                            1
                        }
                    } else {
                        0
                    }
                })
                .sum::<i32>();
            (app.id.clone(), score)
        })
        .collect::<Vec<_>>();

    scored.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    let selected_apps = normalize_route_selection(
        &scored
            .iter()
            .filter(|(_, score)| *score > 0)
            .map(|(app_id, _)| app_id.clone())
            .collect::<Vec<_>>(),
        apps,
        max_apps,
    );
    let risk = max_risk(max_risk_for_selected(&selected_apps, apps), classify_prompt_risk(prompt));

    let top_scores = scored
        .into_iter()
        .filter(|(_, score)| *score > 0)
        .take(3)
        .map(|(app_id, score)| format!("{app_id}:{score}"))
        .collect::<Vec<_>>();

    let rationale = if top_scores.is_empty() {
        if let Some(first) = selected_apps.first() {
            format!("fallback route selected: {first}")
        } else {
            "no route candidates available".to_string()
        }
    } else {
        format!("manifest match route selected ({})", top_scores.join(", "))
    };

    IntentRoute { selected_apps, risk, rationale }
}

#[async_trait]
impl AgentAppHost for InMemoryAgentHost {
    async fn list_apps(&self) -> Vec<AppManifest> {
        let mut apps = self.apps.read().await.clone();
        apps.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
        apps
    }

    async fn route_prompt(&self, prompt: &str) -> IntentRoute {
        let apps = self.apps.read().await.clone();
        if let Some(routed) = route_with_llm(prompt, &apps).await {
            return routed;
        }
        route_with_manifest_similarity(prompt, &apps)
    }

    async fn execute_command(&self, app_id: &str, command: &str) -> CommandDispatch {
        let app_manifest = self.apps.read().await.iter().find(|app| app.id == app_id).cloned();
        let Some(app_manifest) = app_manifest else {
            return CommandDispatch { accepted: false, summary: format!("Unknown app: {app_id}") };
        };

        let output =
            run_app_agent(AppExecutionInput { app: app_manifest, prompt: command.to_string() })
                .await;
        let summary = match (output.model.as_deref(), output.summary.trim()) {
            (Some(model), text) if !text.is_empty() => format!("[{model}] {text}"),
            (_, text) => text.to_string(),
        };
        CommandDispatch { accepted: output.accepted, summary }
    }

    async fn evaluate_handoff_policy(&self, from_app: &str, to_app: &str) -> HandoffPolicyDecision {
        let apps = self.apps.read().await.clone();
        let source_app = apps.iter().find(|app| app.id == from_app);
        if source_app.is_none() {
            return HandoffPolicyDecision {
                allowed: false,
                reason: format!("handoff blocked by policy: unknown source app `{from_app}`"),
            };
        }

        let target_exists = apps.iter().any(|app| app.id == to_app);
        if !target_exists {
            return HandoffPolicyDecision {
                allowed: false,
                reason: format!("handoff blocked by policy: unknown target app `{to_app}`"),
            };
        }

        if from_app == to_app {
            return HandoffPolicyDecision {
                allowed: true,
                reason: "handoff allowed by policy: intra-app transfer".to_string(),
            };
        }

        let source_app = source_app.expect("checked above");
        if source_app.handoff_allowlist.iter().any(|allowed| allowed == to_app) {
            return HandoffPolicyDecision {
                allowed: true,
                reason: format!("handoff allowed by allowlist: {from_app} -> {to_app}"),
            };
        }

        HandoffPolicyDecision {
            allowed: false,
            reason: format!(
                "handoff blocked by allowlist: `{to_app}` is not allowed for `{from_app}`"
            ),
        }
    }

    async fn upsert_app(&self, manifest: AppManifest) -> AppRegistration {
        let mut apps = self.apps.write().await;
        if let Some(existing) = apps.iter_mut().find(|app| app.id == manifest.id) {
            *existing = manifest.clone();
            return AppRegistration { created: false, app_id: manifest.id };
        }
        let app_id = manifest.id.clone();
        apps.push(manifest);
        AppRegistration { created: true, app_id }
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentAppHost, InMemoryAgentHost};

    #[tokio::test]
    async fn route_prompt_prefers_capability_scored_apps() {
        let host = InMemoryAgentHost::default();
        let route = host.route_prompt("schedule a meeting and fix calendar conflicts").await;
        assert!(
            route.selected_apps.iter().any(|app| app == "calendar-agent"),
            "expected calendar-agent in selected apps"
        );
        assert!(
            route.rationale.contains("manifest match route selected")
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

    #[tokio::test]
    async fn handoff_policy_allows_listed_route() {
        let host = InMemoryAgentHost::default();
        let decision = host.evaluate_handoff_policy("ops-center", "mail-agent").await;
        assert!(decision.allowed, "expected route to be allowed");
        assert!(decision.reason.contains("allowlist"));
    }

    #[tokio::test]
    async fn handoff_policy_blocks_unlisted_route() {
        let host = InMemoryAgentHost::default();
        let decision = host.evaluate_handoff_policy("mail-agent", "calendar-agent").await;
        assert!(!decision.allowed, "expected route to be blocked");
        assert!(decision.reason.contains("blocked"));
    }
}
