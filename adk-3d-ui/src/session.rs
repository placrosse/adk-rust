use std::{
    collections::HashMap,
    sync::Arc,
    sync::atomic::{AtomicU64, Ordering},
};

use serde::Serialize;
use tokio::sync::{Mutex, RwLock, broadcast};
use uuid::Uuid;

use crate::protocol::{Seq, SessionId, SseEnvelope, SsePayload, UiEventRequest};

const CHANNEL_CAPACITY: usize = 256;
const MAX_STORED_EVENTS: usize = 128;

#[derive(Debug, Clone)]
pub struct OutboundMessage {
    pub event: String,
    pub data: String,
}

#[derive(Debug, Clone, Default)]
pub struct SessionContext {
    pub last_prompt: Option<String>,
    pub last_command: Option<String>,
    pub selected_id: Option<String>,
    pub last_intent_domain: Option<String>,
    pub last_node_ids: Vec<String>,
}

#[derive(Debug)]
struct SessionState {
    tx: broadcast::Sender<OutboundMessage>,
    server_seq: AtomicU64,
    events: Mutex<Vec<UiEventRequest>>,
    context: RwLock<SessionContext>,
}

impl SessionState {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self {
            tx,
            server_seq: AtomicU64::new(0),
            events: Mutex::new(Vec::new()),
            context: RwLock::new(SessionContext::default()),
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
        let data = match serde_json::to_string(&envelope) {
            Ok(data) => data,
            Err(_) => return None,
        };
        let event = payload.event_name().to_string();
        let _ = state.tx.send(OutboundMessage { event, data });
        Some(seq)
    }

    pub async fn record_event(&self, session_id: &str, event: UiEventRequest) -> Option<()> {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        let mut events = state.events.lock().await;
        events.push(event);
        if events.len() > MAX_STORED_EVENTS {
            let drain_to = events.len() - MAX_STORED_EVENTS;
            events.drain(0..drain_to);
        }
        Some(())
    }

    pub async fn last_server_seq(&self, session_id: &str) -> Option<Seq> {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        Some(state.server_seq.load(Ordering::Relaxed))
    }

    pub async fn has_session(&self, session_id: &str) -> bool {
        let sessions = self.sessions.read().await;
        sessions.contains_key(session_id)
    }

    pub async fn get_context(&self, session_id: &str) -> Option<SessionContext> {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        Some(state.context.read().await.clone())
    }

    pub async fn set_last_prompt(&self, session_id: &str, prompt: String) -> Option<()> {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        state.context.write().await.last_prompt = Some(prompt);
        Some(())
    }

    pub async fn set_last_command(&self, session_id: &str, command: String) -> Option<()> {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        state.context.write().await.last_command = Some(command);
        Some(())
    }

    pub async fn set_selected_id(&self, session_id: &str, selected_id: Option<String>) -> Option<()> {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        state.context.write().await.selected_id = selected_id;
        Some(())
    }

    pub async fn update_plan_state(
        &self,
        session_id: &str,
        intent_domain: String,
        node_ids: Vec<String>,
    ) -> Option<()> {
        let sessions = self.sessions.read().await;
        let state = sessions.get(session_id)?;
        let mut ctx = state.context.write().await;
        ctx.last_intent_domain = Some(intent_domain);
        ctx.last_node_ids = node_ids;
        Some(())
    }
}

pub fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
}
