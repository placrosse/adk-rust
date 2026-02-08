use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{app_runtime::manifest::AppManifest, safety::risk::RiskTier};

pub type SessionId = String;
pub type Seq = u64;
pub type AppId = String;
pub type SurfaceId = String;
pub type UiProps = Map<String, Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEnvelope<T> {
    pub v: String,
    pub seq: Seq,
    pub session: SessionId,
    pub ts: String,
    pub payload: T,
}

impl<T> SseEnvelope<T> {
    pub fn new(seq: Seq, session: SessionId, payload: T) -> Self {
        Self {
            v: "v0".to_string(),
            seq,
            session,
            ts: Utc::now().to_rfc3339(),
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum SsePayload {
    ShellState(ShellStatePayload),
    AppSurfaceOps(AppSurfaceOpsPayload),
    TimelineEntry(TimelineEntryPayload),
    ApprovalRequired(ApprovalRequiredPayload),
    Notification(NotificationPayload),
    Error(ErrorPayload),
    Done(DonePayload),
    Ping(PingPayload),
}

impl SsePayload {
    pub fn event_name(&self) -> &'static str {
        match self {
            Self::ShellState(_) => "shell_state",
            Self::AppSurfaceOps(_) => "app_surface_ops",
            Self::TimelineEntry(_) => "timeline_entry",
            Self::ApprovalRequired(_) => "approval_required",
            Self::Notification(_) => "notification",
            Self::Error(_) => "error",
            Self::Done(_) => "done",
            Self::Ping(_) => "ping",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellStatePayload {
    pub focused_app: Option<AppId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_apps: Vec<AppId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSurfaceOpsPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Seq>,
    pub ops: Vec<SurfaceOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SurfaceOp {
    Create(SurfaceCreateOp),
    Patch(SurfacePatchOp),
    Remove(SurfaceRemoveOp),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceCreateOp {
    pub id: SurfaceId,
    pub app_id: AppId,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub props: UiProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfacePatchOp {
    pub id: SurfaceId,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub props: UiProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceRemoveOp {
    pub id: SurfaceId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntryPayload {
    pub level: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub fields: UiProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequiredPayload {
    pub action_id: String,
    pub app_id: AppId,
    pub title: String,
    pub rationale: String,
    pub risk: RiskTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonePayload {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingPayload {
    pub ts: String,
}

impl PingPayload {
    pub fn now() -> Self {
        Self {
            ts: Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreateResponse {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterPromptRequest {
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterPromptResponse {
    pub accepted: bool,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_apps: Vec<AppId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundEventRequest {
    pub seq: Seq,
    pub event: InboundEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InboundEvent {
    MasterPromptSubmit { prompt: String },
    AppFocus { app_id: AppId },
    AppCommand { app_id: AppId, command: String },
    ApprovalDecision { action_id: String, approved: bool },
    WorkspaceLayoutChange { layout: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundEventAck {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_seq: Option<Seq>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCatalogResponse {
    pub apps: Vec<AppManifest>,
}
