use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Safe,
    Controlled,
    Dangerous,
}

pub fn risk_rank(risk: RiskTier) -> usize {
    match risk {
        RiskTier::Safe => 0,
        RiskTier::Controlled => 1,
        RiskTier::Dangerous => 2,
    }
}

pub fn max_risk(left: RiskTier, right: RiskTier) -> RiskTier {
    if risk_rank(left) >= risk_rank(right) { left } else { right }
}

/// Generic prompt-level risk heuristic used as a fallback when model-based
/// policy output is unavailable.
pub fn classify_prompt_risk(prompt: &str) -> RiskTier {
    let normalized = prompt.to_lowercase();
    let dangerous_terms = [
        "rollback",
        "restart",
        "shutdown",
        "terminate",
        "delete",
        "drop",
        "wipe",
        "scale down",
        "revoke",
    ];
    if dangerous_terms.iter().any(|term| normalized.contains(term)) {
        return RiskTier::Dangerous;
    }

    let controlled_terms =
        ["create", "update", "send", "approve", "schedule", "handoff", "trigger"];
    if controlled_terms.iter().any(|term| normalized.contains(term)) {
        return RiskTier::Controlled;
    }

    RiskTier::Safe
}
