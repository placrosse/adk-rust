use std::{
    collections::HashMap,
    sync::Arc,
    sync::atomic::{AtomicU64, Ordering},
};

use tokio::sync::{Mutex, RwLock, broadcast};
use uuid::Uuid;

use crate::{
    app_runtime::handoff::PendingHandoff,
    protocol::{InboundEventRequest, Seq, SessionId, SseEnvelope, SsePayload},
    safety::{approvals::PendingApproval, audit::AuditEntry},
};

const CHANNEL_CAPACITY: usize = 256;
const MAX_EVENT_HISTORY: usize = 128;

#[derive(Debug, Clone)]
pub struct OutboundMessage {
    pub event: String,
    pub data: String,
}

#[derive(Debug, Clone, Default)]
pub struct AppSurfaceLayout {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub z_index: i32,
}

#[derive(Debug, Clone, Default)]
pub struct ShellSessionContext {
    pub focused_app: Option<String>,
    pub active_apps: Vec<String>,
    pub last_prompt: Option<String>,
    pub workspace_layout: HashMap<String, AppSurfaceLayout>,
    pub pending_approval: Option<PendingApproval>,
    pub pending_handoff: Option<PendingHandoff>,
    pub audit_log: Vec<AuditEntry>,
}

#[derive(Debug)]
struct SessionState {
    tx: broadcast::Sender<OutboundMessage>,
    server_seq: AtomicU64,
    inbound_events: Mutex<Vec<InboundEventRequest>>,
    context: RwLock<ShellSessionContext>,
}

impl SessionState {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self {
            tx,
            server_seq: AtomicU64::new(0),
            inbound_events: Mutex::new(Vec::new()),
            context: RwLock::new(ShellSessionContext::default()),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Arc<SessionState>>>>,
}

impl SessionManager {
    pub async fn create_session(&self) -> SessionId {
        let session_id = Uuid::new_v4().to_string();
        let state = Arc::new(SessionState::new());
        self.sessions.write().await.insert(session_id.clone(), state);
        session_id
    }

    pub async fn ensure_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        if !sessions.contains_key(session_id) {
            sessions.insert(session_id.to_string(), Arc::new(SessionState::new()));
        }
    }

    pub async fn subscribe(&self, session_id: &str) -> Option<broadcast::Receiver<OutboundMessage>> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|state| state.tx.subscribe())
    }

    pub async fn publish(&self, session_id: &str, payload: SsePayload) -> Option<Seq> {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        let seq = state.server_seq.fetch_add(1, Ordering::Relaxed) + 1;
        let envelope = SseEnvelope::new(seq, session_id.to_string(), payload.clone());
        let data = serde_json::to_string(&envelope).ok()?;
        let event = payload.event_name().to_string();
        let _ = state.tx.send(OutboundMessage { event, data });
        Some(seq)
    }

    pub async fn record_event(&self, session_id: &str, event: InboundEventRequest) -> Option<()> {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        let mut events = state.inbound_events.lock().await;
        events.push(event);
        if events.len() > MAX_EVENT_HISTORY {
            let drain_to = events.len() - MAX_EVENT_HISTORY;
            events.drain(0..drain_to);
        }
        Some(())
    }

    pub async fn last_server_seq(&self, session_id: &str) -> Option<Seq> {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        Some(state.server_seq.load(Ordering::Relaxed))
    }

    pub async fn get_context(&self, session_id: &str) -> Option<ShellSessionContext> {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        Some(state.context.read().await.clone())
    }

    pub async fn update_context<F>(&self, session_id: &str, mutator: F) -> Option<()>
    where
        F: FnOnce(&mut ShellSessionContext),
    {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        let mut context = state.context.write().await;
        mutator(&mut context);
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::SessionManager;

    #[tokio::test]
    async fn context_updates_round_trip() {
        let sessions = SessionManager::default();
        let session_id = sessions.create_session().await;

        let _ = sessions
            .update_context(&session_id, |ctx| {
                ctx.last_prompt = Some("hello".to_string());
                ctx.focused_app = Some("ops-center".to_string());
            })
            .await;

        let context = sessions
            .get_context(&session_id)
            .await
            .expect("context exists");
        assert_eq!(context.last_prompt.as_deref(), Some("hello"));
        assert_eq!(context.focused_app.as_deref(), Some("ops-center"));
    }
}
