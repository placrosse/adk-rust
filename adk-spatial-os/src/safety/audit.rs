use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::risk::RiskTier;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditDecision {
    Proposed,
    Approved,
    Rejected,
    Executed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub action_id: String,
    pub app_id: String,
    pub risk: RiskTier,
    pub decision: AuditDecision,
    pub ts: String,
}

impl AuditEntry {
    pub fn new(action_id: &str, app_id: &str, risk: RiskTier, decision: AuditDecision) -> Self {
        Self {
            action_id: action_id.to_string(),
            app_id: app_id.to_string(),
            risk,
            decision,
            ts: Utc::now().to_rfc3339(),
        }
    }
}
