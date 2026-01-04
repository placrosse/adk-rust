#[cfg(test)]
mod tests {
    use crate::memory::{SpanData, SpanStatus};
    use serde_json;

    #[test]
    fn test_span_data_serialization() {
        let span = SpanData {
            id: "test123".to_string(),
            trace_id: "trace456".to_string(),
            name: "test_span".to_string(),
            parent_id: None,
            start_time: 1234567890000000000,
            end_time: Some(1234567890000001000),
            kind: 1,
            attributes: std::collections::HashMap::new(),
            status: SpanStatus { code: 1, message: None },
            invoc_id: "test123".to_string(),
        };
        
        let json = serde_json::to_string_pretty(&span).unwrap();
        println!("Serialized SpanData:\n{}", json);
        
        // Check that it has the correct field names
        assert!(json.contains("\"span_id\""));
        assert!(json.contains("\"trace_id\""));
        assert!(json.contains("\"start_time\""));
        assert!(json.contains("\"end_time\""));
        
        // Should NOT contain OTLP field names
        assert!(!json.contains("\"spanId\""));
        assert!(!json.contains("\"startTimeUnixNano\""));
    }
}
