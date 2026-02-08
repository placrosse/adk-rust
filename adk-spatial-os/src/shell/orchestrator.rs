use crate::{
    app_runtime::host::{AgentAppHost, IntentRoute},
    safety::risk::RiskTier,
};

#[derive(Debug, Clone)]
pub struct MasterPlan {
    pub prompt: String,
    pub selected_apps: Vec<String>,
    pub risk: RiskTier,
    pub rationale: String,
}

pub async fn build_master_plan(host: &dyn AgentAppHost, prompt: &str) -> MasterPlan {
    let IntentRoute {
        selected_apps,
        risk,
        rationale,
    } = host.route_prompt(prompt).await;

    MasterPlan {
        prompt: prompt.to_string(),
        selected_apps,
        risk,
        rationale,
    }
}
