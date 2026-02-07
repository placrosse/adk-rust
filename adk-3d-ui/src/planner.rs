use serde::{Deserialize, Serialize};

use crate::policy::{ProposedAction, RiskTier, classify_action};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentDomain {
    Ops,
    Incident,
    Inventory,
    Greeting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntentUrgency {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptIntent {
    pub domain: IntentDomain,
    pub urgency: IntentUrgency,
    pub focus_hint: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Default)]
pub struct PlanningContext {
    pub last_prompt: Option<String>,
    pub last_command: Option<String>,
    pub selected_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbNode {
    pub id: String,
    pub label: String,
    pub status: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenePlan {
    pub intent: PromptIntent,
    pub title: String,
    pub subtitle: String,
    pub workbench_title: String,
    pub workbench_summary: String,
    pub prompt_echo: String,
    pub nodes: Vec<OrbNode>,
    pub action: Option<ProposedAction>,
}

pub fn parse_intent(prompt: &str) -> PromptIntent {
    let normalized = prompt.trim();
    let lower = normalized.to_lowercase();

    let domain = if lower.contains("inventory") || lower.contains("stock") {
        IntentDomain::Inventory
    } else if lower.contains("incident")
        || lower.contains("outage")
        || lower.contains("degraded")
        || lower.contains("blast radius")
    {
        IntentDomain::Incident
    } else if lower.contains("hello world") || lower.contains("greeting") {
        IntentDomain::Greeting
    } else {
        IntentDomain::Ops
    };

    let urgency = if lower.contains("now")
        || lower.contains("urgent")
        || lower.contains("critical")
        || lower.contains("immediately")
    {
        IntentUrgency::High
    } else if lower.contains("today") || lower.contains("soon") {
        IntentUrgency::Medium
    } else {
        IntentUrgency::Low
    };

    let focus_hint = if lower.contains("payments") {
        Some("service-payments".to_string())
    } else if lower.contains("checkout") {
        Some("service-checkout".to_string())
    } else if lower.contains("identity") {
        Some("service-identity".to_string())
    } else {
        None
    };

    PromptIntent {
        domain,
        urgency,
        focus_hint,
        summary: normalized.to_string(),
    }
}

pub fn build_scene_plan(prompt: &str, context: &PlanningContext) -> ScenePlan {
    let normalized = prompt.trim();
    let lower = normalized.to_lowercase();
    let intent = parse_intent(prompt);

    let (title, subtitle) = if matches!(intent.domain, IntentDomain::Inventory) {
        (
            "Inventory Orbit".to_string(),
            "Stock posture across critical SKUs".to_string(),
        )
    } else if matches!(intent.domain, IntentDomain::Incident) {
        (
            "Incident Constellation".to_string(),
            "Active risk and blast radius at a glance".to_string(),
        )
    } else if matches!(intent.domain, IntentDomain::Greeting) {
        (
            "HELLO WORLD".to_string(),
            "A living 3D greeting scene".to_string(),
        )
    } else {
        (
            "Orbital Ops".to_string(),
            "System health and next-best actions".to_string(),
        )
    };

    let mut nodes = match intent.domain {
        IntentDomain::Inventory => vec![
            OrbNode {
                id: "sku-a123".to_string(),
                label: "SKU A123".to_string(),
                status: "warning".to_string(),
                x: -2.4,
                y: 0.6,
                z: -6.0,
            },
            OrbNode {
                id: "sku-b456".to_string(),
                label: "SKU B456".to_string(),
                status: "critical".to_string(),
                x: 0.0,
                y: 1.2,
                z: -5.0,
            },
            OrbNode {
                id: "sku-c777".to_string(),
                label: "SKU C777".to_string(),
                status: "healthy".to_string(),
                x: 2.4,
                y: 0.5,
                z: -6.3,
            },
        ],
        IntentDomain::Incident => vec![
            OrbNode {
                id: "service-payments".to_string(),
                label: "Payments".to_string(),
                status: "critical".to_string(),
                x: -2.4,
                y: 0.6,
                z: -6.0,
            },
            OrbNode {
                id: "service-checkout".to_string(),
                label: "Checkout".to_string(),
                status: "degraded".to_string(),
                x: 0.0,
                y: 1.2,
                z: -5.0,
            },
            OrbNode {
                id: "service-identity".to_string(),
                label: "Identity".to_string(),
                status: "warning".to_string(),
                x: 2.4,
                y: 0.5,
                z: -6.3,
            },
        ],
        IntentDomain::Greeting => vec![
            OrbNode {
                id: "hello-left".to_string(),
                label: "HELLO".to_string(),
                status: "healthy".to_string(),
                x: -1.6,
                y: 0.9,
                z: -5.6,
            },
            OrbNode {
                id: "hello-right".to_string(),
                label: "WORLD".to_string(),
                status: "healthy".to_string(),
                x: 1.6,
                y: 0.9,
                z: -5.6,
            },
        ],
        IntentDomain::Ops => vec![
            OrbNode {
                id: "service-payments".to_string(),
                label: "Payments".to_string(),
                status: "degraded".to_string(),
                x: -2.4,
                y: 0.6,
                z: -6.0,
            },
            OrbNode {
                id: "service-checkout".to_string(),
                label: "Checkout".to_string(),
                status: "healthy".to_string(),
                x: 0.0,
                y: 1.2,
                z: -5.0,
            },
            OrbNode {
                id: "service-identity".to_string(),
                label: "Identity".to_string(),
                status: "warning".to_string(),
                x: 2.4,
                y: 0.5,
                z: -6.3,
            },
        ],
    };

    let selected_or_intent = intent.focus_hint.clone().or_else(|| context.selected_id.clone());
    if let Some(focus_id) = selected_or_intent {
        for node in &mut nodes {
            if node.id == focus_id {
                node.y += 0.55;
                node.z += 0.8;
                node.status = "focused".to_string();
            }
        }
    }

    let risk = classify_action(&lower);
    let action = match risk {
        RiskTier::Safe => None,
        RiskTier::Controlled | RiskTier::Dangerous => Some(ProposedAction {
            action_id: "action-1".to_string(),
            label: if matches!(risk, RiskTier::Dangerous) {
                "Execute high-risk remediation".to_string()
            } else {
                "Create and annotate incident".to_string()
            },
            rationale: "Action inferred from prompt keywords.".to_string(),
            risk,
            requires_approval: matches!(risk, RiskTier::Dangerous),
        }),
    };

    let subtitle = if let Some(selected) = &context.selected_id {
        format!("{subtitle} | Focused: {selected}")
    } else {
        subtitle
    };

    let subtitle = if let Some(last_command) = &context.last_command {
        format!("{subtitle} | Command: {last_command}")
    } else {
        subtitle
    };

    let subtitle = match intent.urgency {
        IntentUrgency::High => format!("{subtitle} | Urgency: high"),
        IntentUrgency::Medium => format!("{subtitle} | Urgency: medium"),
        IntentUrgency::Low => subtitle,
    };

    let workbench_title = if let Some(selected) = &context.selected_id {
        format!("Service Workbench: {selected}")
    } else {
        "Service Workbench".to_string()
    };

    let workbench_summary = if let Some(last_prompt) = &context.last_prompt {
        format!("Last request: {last_prompt}")
    } else {
        "Select a node to inspect deployment and incident context.".to_string()
    };

    ScenePlan {
        intent,
        title,
        subtitle,
        workbench_title,
        workbench_summary,
        prompt_echo: normalized.to_string(),
        nodes,
        action,
    }
}

#[cfg(test)]
mod tests {
    use super::{IntentDomain, PlanningContext, build_scene_plan, parse_intent};

    #[test]
    fn hello_world_prompt_uses_hello_title() {
        let plan = build_scene_plan("Hello world in 3d", &PlanningContext::default());
        assert_eq!(plan.title, "HELLO WORLD");
    }

    #[test]
    fn dangerous_prompt_produces_action() {
        let plan = build_scene_plan("rollback payments now", &PlanningContext::default());
        assert!(plan.action.is_some());
        assert!(plan.action.unwrap().requires_approval);
    }

    #[test]
    fn parse_intent_detects_domain() {
        let intent = parse_intent("show me inventory risk");
        assert_eq!(intent.domain, IntentDomain::Inventory);
    }
}
