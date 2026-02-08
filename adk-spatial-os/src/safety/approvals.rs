use serde::{Deserialize, Serialize};

use super::risk::RiskTier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub action_id: String,
    pub app_id: String,
    pub title: String,
    pub rationale: String,
    pub risk: RiskTier,
}
