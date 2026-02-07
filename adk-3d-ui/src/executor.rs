use serde_json::{Map, json};

use crate::{planner::ScenePlan, protocol::{UiCreateOp, UiOp, UiOpsPayload, UiPatchOp, UiProps}};

fn props(value: serde_json::Value) -> UiProps {
    value.as_object().cloned().unwrap_or_else(Map::new)
}

pub fn scene_plan_to_ops(plan: &ScenePlan, reply_to: Option<u64>) -> UiOpsPayload {
    let mut ops = Vec::new();

    ops.push(UiOp::Create(UiCreateOp {
        id: "root".to_string(),
        kind: "group".to_string(),
        parent: None,
        props: props(json!({"x":0.0,"y":0.0,"z":0.0})),
    }));

    ops.push(UiOp::Create(UiCreateOp {
        id: "title".to_string(),
        kind: "text3d".to_string(),
        parent: Some("root".to_string()),
        props: props(json!({
            "text": plan.title,
            "x": 0.0,
            "y": 2.1,
            "z": -5.2,
            "size": 0.42,
            "color": "#F5F5FF"
        })),
    }));

    ops.push(UiOp::Create(UiCreateOp {
        id: "subtitle".to_string(),
        kind: "text3d".to_string(),
        parent: Some("root".to_string()),
        props: props(json!({
            "text": plan.subtitle,
            "x": 0.0,
            "y": 1.45,
            "z": -5.4,
            "size": 0.14,
            "color": "#95A2C6"
        })),
    }));

    for node in &plan.nodes {
        ops.push(UiOp::Create(UiCreateOp {
            id: node.id.clone(),
            kind: "orb".to_string(),
            parent: Some("root".to_string()),
            props: props(json!({
                "label": node.label,
                "status": node.status,
                "x": node.x,
                "y": node.y,
                "z": node.z,
                "radius": 0.28
            })),
        }));
    }

    if let Some(action) = &plan.action {
        ops.push(UiOp::Create(UiCreateOp {
            id: "action-card".to_string(),
            kind: "panel3d".to_string(),
            parent: Some("root".to_string()),
            props: props(json!({
                "x": 0.0,
                "y": -1.1,
                "z": -4.8,
                "w": 4.4,
                "h": 1.4,
                "title": action.label,
                "subtitle": action.rationale,
                "requiresApproval": action.requires_approval,
                "risk": action.risk,
                "actionId": action.action_id
            })),
        }));
    }

    ops.push(UiOp::Patch(UiPatchOp {
        id: "root".to_string(),
        props: props(json!({
            "promptEcho": plan.prompt_echo,
            "generatedAt": chrono::Utc::now().to_rfc3339(),
        })),
    }));

    UiOpsPayload { reply_to, ops }
}

#[cfg(test)]
mod tests {
    use crate::planner::{PlanningContext, build_scene_plan};

    use super::scene_plan_to_ops;

    #[test]
    fn scene_plan_generates_multiple_ops() {
        let plan = build_scene_plan("show me platform health", &PlanningContext::default());
        let payload = scene_plan_to_ops(&plan, Some(2));
        assert!(payload.ops.len() >= 6);
        assert_eq!(payload.reply_to, Some(2));
    }
}
