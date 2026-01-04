use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tracing::{Id, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

/// Data for a captured span
#[derive(Debug, Clone, Serialize)]
pub struct SpanData {
    #[serde(rename = "span_id")]
    pub id: String,
    #[serde(rename = "trace_id")]
    pub trace_id: String,
    pub name: String,
    #[serde(rename = "parent_span_id", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    
    // Original ADK format: nanoseconds as numbers
    pub start_time: u128,
    pub end_time: Option<u128>,
    
    // OTLP Kind:
    // 0=Unspecified, 1=Internal, 2=Server, 3=Client, 4=Producer, 5=Consumer
    pub kind: i32,
    
    pub attributes: HashMap<String, serde_json::Value>,
    pub status: SpanStatus,
    
    // UI compatibility field
    pub invoc_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpanStatus {
    // 0=Unset, 1=Ok, 2=Error
    pub code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Shared storage for traces
#[derive(Debug, Clone, Default)]
pub struct SharedTraceStorage {
    /// Map of event_id (or invocation_id) -> List of spans
    traces: Arc<RwLock<HashMap<String, Vec<SpanData>>>>,
    /// Map of alias -> real_key (e.g. event_id -> invocation_id)
    aliases: Arc<RwLock<HashMap<String, String>>>,
}

impl SharedTraceStorage {
    pub fn new() -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
            aliases: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_trace(&self, key: &str) -> Option<Vec<SpanData>> {
        eprintln!("DEBUG: SharedTraceStorage::get_trace called with key: {}", key);
        // Resolve alias if exists
        let real_key = if let Ok(aliases) = self.aliases.read() {
            aliases.get(key).cloned().unwrap_or_else(|| key.to_string())
        } else {
            key.to_string()
        };

        let result = self.traces.read().ok()?.get(&real_key).cloned();
        eprintln!("DEBUG: get_trace result for key '{}': {:?}", key, result.as_ref().map(|v| v.len()));
        result
    }

    pub fn add_span(&self, key: String, span: SpanData) {
        eprintln!("DEBUG: SharedTraceStorage::add_span called with key: {}", key);
        if let Ok(mut traces) = self.traces.write() {
            traces.entry(key.clone()).or_default().push(span);
            eprintln!("DEBUG: Span added to storage, total keys: {}", traces.len());
        } else {
            eprintln!("DEBUG: Failed to acquire write lock on traces");
        }
    }

    pub fn add_alias(&self, alias: String, key: String) {
        if let Ok(mut aliases) = self.aliases.write() {
            aliases.insert(alias, key);
        }
    }
}

/// A tracing layer that captures spans in memory
pub struct InMemoryTraceLayer {
    storage: Arc<SharedTraceStorage>,
}

impl InMemoryTraceLayer {
    pub fn new(storage: Arc<SharedTraceStorage>) -> Self {
        Self { storage }
    }
}

#[derive(Clone)]
struct SpanFields(HashMap<String, serde_json::Value>);

impl<S> Layer<S> for InMemoryTraceLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &tracing::span::Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found");
        let mut extensions = span.extensions_mut();
        
        // Capture start time - always set it, don't check if it exists
        let start = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128; // Use nanoseconds directly
        extensions.insert(start);

        // Capture fields
        let mut visitor = JsonVisitor::default();
        attrs.record(&mut visitor);
        let mut fields_map = visitor.0;

        // Propagate fields from parent span
        if let Some(parent) = span.parent() {
            if let Some(parent_fields) = parent.extensions().get::<SpanFields>() {
                // List of keys to propagate for context
                let context_keys = [
                    "session.id", "session_id", 
                    "invocation.id", "invocation_id", 
                    "event_id"
                ];

                for key in context_keys {
                     // Only propagate if not overridden by the current span
                     if !fields_map.contains_key(key) {
                         if let Some(val) = parent_fields.0.get(key) {
                             fields_map.insert(key.to_string(), val.clone());
                         }
                     }
                }
            }
        }

        extensions.insert(SpanFields(fields_map));
    }

    fn on_record(&self, id: &Id, values: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found");
        let mut extensions = span.extensions_mut();
        if let Some(fields) = extensions.get_mut::<SpanFields>() {
            let mut visitor = JsonVisitor::default();
            values.record(&mut visitor);
            for (k, v) in visitor.0 {
                fields.0.insert(k, v);
            }
        }
    }

    // Capture events (logs) too, to catch correlation events
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = JsonVisitor::default();
        event.record(&mut visitor);
        let fields = visitor.0;
        
        // Check for correlation event
        let event_id = fields.get("event_id").and_then(|v| v.as_str()).map(|s| s.to_string());
        let inv_id = fields.get("invocation_id").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        if let (Some(eid), Some(iid)) = (event_id, inv_id) {
            self.storage.add_alias(eid, iid);
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).expect("Span not found");
        let extensions = span.extensions();
        let start_time_nanos = extensions.get::<u128>().copied().unwrap_or(0);
        
        let end_time_nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u128;

        // Extract metadata
        let metadata = span.metadata();
        let name = metadata.name().to_string();
        let span_name_for_debug = name.clone();

        // Retrieve captured fields
        let mut fields = extensions.get::<SpanFields>().map(|f| f.0.clone()).unwrap_or_default();
        
        // Find trace identifiers
        let mut keys = Vec::new();
        let mut trace_id = String::new(); 
        
        // Invocation ID (primary candidate for trace_id)
        if let Some(data) = fields.get("invocation.id").or_else(|| fields.get("invocation_id")) {
             if let Some(s) = data.as_str() {
                 let s_str = s.to_string();
                 keys.push(s_str.clone());
                 trace_id = s_str; // Use invocation_id as trace_id
             }
        }
        
        // Session ID
        if let Some(data) = fields.get("session.id").or_else(|| fields.get("session_id")) {
             if let Some(s) = data.as_str() {
                 keys.push(s.to_string());
             }
        }
        
        // Event ID
        if let Some(data) = fields.get("event_id").and_then(|v| v.as_str()) {
            keys.push(data.to_string());
            if trace_id.is_empty() {
                trace_id = data.to_string(); // Fallback to event_id
            }
        }
        
        // IF trace_id is still empty, generate a random one
        if trace_id.is_empty() || trace_id.chars().all(|c| c == '0') {
             // Generate random 128-bit hex string from timestamp + span_id
             let r1 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
             let r2 = id.into_u64();
             trace_id = format!("{:016x}{:016x}", r1, r2); 
        }

        if keys.is_empty() {
             // Add session_id as fallback key if no other keys found
             if let Some(sess_id) = fields.get("session_id").and_then(|v| v.as_str()) {
                 keys.push(sess_id.to_string());
             } else {
                 return; // No keys to store under
             }
        }
        
        // Ensure camelCase keys are present in attributes for UI (backward compat)
        if let Some(inv_id) = fields.get("invocation_id").cloned() {
            fields.insert("invocationId".to_string(), inv_id.clone());
            fields.insert("gcp.vertex.agent.invocation_id".to_string(), inv_id);
        } else {
             // Fallback for UI crash prevention: use trace_id as invocation_id proxy
             let proxy_id = serde_json::Value::String(trace_id.clone());
             fields.insert("gcp.vertex.agent.invocation_id".to_string(), proxy_id);
        }
        if let Some(sess_id) = fields.get("session_id").cloned() {
            fields.insert("sessionId".to_string(), sess_id);
        }

        // Create span data once
        let _final_invoc_id = if trace_id.is_empty() { 
            Some(format!("span-{:016x}", id.into_u64()))
        } else { 
            Some(trace_id.clone()) 
        };
        
        let span_data = SpanData {
            id: format!("{:016x}", id.into_u64()), // Hex span ID (padded)
            trace_id: trace_id.clone(),
            name,
            parent_id: span.parent().map(|p| format!("{:016x}", p.id().into_u64())), // Hex parent ID
            
            start_time: start_time_nanos,
            end_time: Some(end_time_nanos),
            
            kind: 1, // INTERNAL
            status: SpanStatus { code: 1, message: None }, // OK
            
            attributes: fields,
            invoc_id: trace_id.clone(),
        };
        
        // Store under all keys
        for key in keys {
            eprintln!("DEBUG: Storing span '{}' under key '{}'", span_name_for_debug, key);
            self.storage.add_span(key, span_data.clone());
        }
        eprintln!("DEBUG: Span storage complete");
    }
}

#[derive(Default)]
struct JsonVisitor(HashMap<String, serde_json::Value>);

impl tracing::field::Visit for JsonVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(field.name().to_string(), serde_json::Value::String(format!("{:?}", value)));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.insert(field.name().to_string(), serde_json::Value::String(value.to_string()));
    }
    
    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0.insert(field.name().to_string(), serde_json::Value::Bool(value));
    }
    
    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0.insert(field.name().to_string(), serde_json::json!(value));
    }
}
