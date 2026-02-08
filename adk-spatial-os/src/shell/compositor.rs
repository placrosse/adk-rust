use std::collections::HashMap;

use serde_json::json;

use crate::app_runtime::manifest::AppManifest;
use crate::protocol::{
    AppSurfaceOpsPayload, ShellStatePayload, SurfaceCreateOp, SurfaceOp, TimelineEntryPayload,
};
use crate::session::AppSurfaceLayout;

pub fn shell_state(
    selected_apps: Vec<String>,
    focused_app: Option<String>,
    last_prompt: Option<String>,
) -> ShellStatePayload {
    ShellStatePayload { focused_app, active_apps: selected_apps, last_prompt }
}

pub fn build_app_surface_ops(
    selected_apps: &[String],
    workspace_layout: &HashMap<String, AppSurfaceLayout>,
    app_catalog: &HashMap<String, AppManifest>,
) -> AppSurfaceOpsPayload {
    let ops = selected_apps
        .iter()
        .enumerate()
        .map(|(idx, app_id)| {
            let manifest = app_catalog.get(app_id);
            let title = manifest.map(|app| app.name.as_str()).unwrap_or("Agent App");
            let saved_layout = workspace_layout.get(app_id);
            SurfaceOp::Create(SurfaceCreateOp {
                id: format!("surface:{}", app_id),
                app_id: app_id.clone(),
                kind: "window".to_string(),
                props: json!({
                    "title": title,
                    "x": saved_layout.map(|layout| layout.x).unwrap_or(80 + (idx as i32 * 120)),
                    "y": saved_layout.map(|layout| layout.y).unwrap_or(90 + (idx as i32 * 48)),
                    "w": saved_layout.map(|layout| layout.w).unwrap_or(520),
                    "h": saved_layout.map(|layout| layout.h).unwrap_or(320),
                    "z_index": saved_layout.map(|layout| layout.z_index).unwrap_or(10 + idx as i32),
                    "content": manifest
                        .map(|app| app.description.clone())
                        .filter(|description| !description.trim().is_empty())
                        .unwrap_or_else(|| format!("{} ready for execution.", title)),
                })
                .as_object()
                .cloned()
                .unwrap_or_default(),
            })
        })
        .collect();

    AppSurfaceOpsPayload { reply_to: None, ops }
}

pub fn timeline_info(message: &str) -> TimelineEntryPayload {
    TimelineEntryPayload {
        level: "info".to_string(),
        message: message.to_string(),
        fields: Default::default(),
    }
}
