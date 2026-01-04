use adk_telemetry::memory::{SpanData, SpanStatus};
use std::collections::HashMap;

fn main() {
    let span = SpanData {
        id: "test123".to_string(),
        trace_id: "trace456".to_string(),
        name: "test_span".to_string(),
        parent_id: None,
        start_time: 1234567890000000000,
        end_time: Some(1234567890000001000),
        kind: 1,
        attributes: HashMap::new(),
        status: SpanStatus { code: 1, message: None },
    };
    
    let json = serde_json::to_string_pretty(&span).unwrap();
    println!("{}", json);
}
