use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Safe,
    Controlled,
    Dangerous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedAction {
    pub action_id: String,
    pub label: String,
    pub rationale: String,
    pub risk: RiskTier,
    pub requires_approval: bool,
}

pub fn classify_action(prompt: &str) -> RiskTier {
    let p = prompt.to_lowercase();
    if p.contains("restart") || p.contains("rollback") || p.contains("scale down") {
        RiskTier::Dangerous
    } else if p.contains("ack") || p.contains("acknowledge") || p.contains("incident") {
        RiskTier::Controlled
    } else {
        RiskTier::Safe
    }
}
