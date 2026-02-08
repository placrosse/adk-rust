use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Safe,
    Controlled,
    Dangerous,
}

pub fn classify_prompt_risk(prompt: &str) -> RiskTier {
    let p = prompt.to_lowercase();
    if p.contains("rollback") || p.contains("restart") || p.contains("scale down") {
        RiskTier::Dangerous
    } else if p.contains("create") || p.contains("ack") || p.contains("approve") {
        RiskTier::Controlled
    } else {
        RiskTier::Safe
    }
}
