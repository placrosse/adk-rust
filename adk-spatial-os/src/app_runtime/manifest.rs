use serde::{Deserialize, Serialize};

use crate::safety::risk::RiskTier;

fn default_runtime_mode() -> String {
    "adk_runner".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRuntimeConfig {
    #[serde(default = "default_runtime_mode")]
    pub mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    #[serde(default)]
    pub supports_sub_agents: bool,
    #[serde(default)]
    pub supports_a2a: bool,
}

impl Default for AppRuntimeConfig {
    fn default() -> Self {
        Self {
            mode: default_runtime_mode(),
            workflow_type: None,
            root_agent_id: None,
            default_model: None,
            provider: None,
            instruction: None,
            supports_sub_agents: false,
            supports_a2a: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handoff_allowlist: Vec<String>,
    pub default_risk: RiskTier,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub starter_prompts: Vec<String>,
    #[serde(default)]
    pub runtime: AppRuntimeConfig,
}

pub fn default_manifests() -> Vec<AppManifest> {
    vec![
        AppManifest {
            id: "ops-center".to_string(),
            name: "Ops Center".to_string(),
            version: "0.1.0".to_string(),
            description: "Service health, incidents, and runbook remediation".to_string(),
            capabilities: vec!["ops".to_string(), "incident".to_string(), "remediation".to_string()],
            permissions: vec!["read.telemetry".to_string(), "write.remediation".to_string()],
            handoff_allowlist: vec!["mail-agent".to_string(), "calendar-agent".to_string()],
            default_risk: RiskTier::Controlled,
            starter_prompts: vec![
                "Identify top degraded services and propose safest mitigations.".to_string(),
                "Draft an incident timeline with owners, impact, and checkpoints.".to_string(),
                "Summarize platform health for executive update in three bullets.".to_string(),
            ],
            runtime: AppRuntimeConfig {
                default_model: Some("gemini-2.5-flash".to_string()),
                instruction: Some(
                    "You are the Ops Center agent. Analyze reliability signals, explain risk, and propose low-blast-radius remediation plans.".to_string(),
                ),
                ..AppRuntimeConfig::default()
            },
        },
        AppManifest {
            id: "mail-agent".to_string(),
            name: "Mail Agent".to_string(),
            version: "0.1.0".to_string(),
            description: "Inbox triage, drafting, and follow-ups".to_string(),
            capabilities: vec!["email".to_string(), "summarize".to_string()],
            permissions: vec!["read.mail".to_string(), "send.mail".to_string()],
            handoff_allowlist: vec!["ops-center".to_string()],
            default_risk: RiskTier::Controlled,
            starter_prompts: vec![
                "Summarize unread operational mail and extract urgent action items.".to_string(),
                "Draft a customer-safe incident update with current status and ETA.".to_string(),
                "Create concise follow-ups for open incident threads with owners.".to_string(),
            ],
            runtime: AppRuntimeConfig {
                default_model: Some("gemini-2.5-flash".to_string()),
                instruction: Some(
                    "You are the Mail Agent. Triage inbound messages, draft clear responses, and keep communication concise and actionable.".to_string(),
                ),
                ..AppRuntimeConfig::default()
            },
        },
        AppManifest {
            id: "calendar-agent".to_string(),
            name: "Calendar Agent".to_string(),
            version: "0.1.0".to_string(),
            description: "Scheduling, conflict resolution, and prep briefs".to_string(),
            capabilities: vec!["calendar".to_string(), "scheduling".to_string()],
            permissions: vec!["read.calendar".to_string(), "write.calendar".to_string()],
            handoff_allowlist: vec!["ops-center".to_string()],
            default_risk: RiskTier::Safe,
            starter_prompts: vec![
                "Find a 30-minute overlap for SRE, backend, and support today.".to_string(),
                "Build a mitigation meeting agenda with dependencies and prep tasks.".to_string(),
                "Resolve schedule conflicts while preserving critical attendees.".to_string(),
            ],
            runtime: AppRuntimeConfig {
                default_model: Some("gemini-2.5-flash".to_string()),
                instruction: Some(
                    "You are the Calendar Agent. Coordinate scheduling decisions, resolve conflicts, and produce practical meeting plans.".to_string(),
                ),
                ..AppRuntimeConfig::default()
            },
        },
    ]
}
