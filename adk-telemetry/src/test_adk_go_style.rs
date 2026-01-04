use std::sync::Arc;
use tracing::{info_span, Instrument};
use crate::{init_with_adk_exporter, AdkSpanExporter};

#[tokio::test]
async fn test_adk_go_style_tracing() {
    // Initialize telemetry with ADK-Go style exporter
    let exporter = init_with_adk_exporter("test-service").expect("Failed to init telemetry");
    
    // Create a span with ADK-Go style attributes
    let span = info_span!(
        "agent.execute",
        "gcp.vertex.agent.invocation_id" = "test-inv-123",
        "gcp.vertex.agent.session_id" = "test-session-456", 
        "gcp.vertex.agent.event_id" = "test-event-789",
        "agent.name" = "test-agent"
    );
    
    // Execute some work in the span
    async {
        tracing::info!("Agent execution started");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        tracing::info!("Agent execution completed");
    }
    .instrument(span)
    .await;
    
    // Small delay to ensure span is processed
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    // Verify span was captured by event_id
    let trace_by_event_id = exporter.get_trace_by_event_id("test-event-789");
    assert!(trace_by_event_id.is_some(), "Should capture span by event_id");
    
    let attributes = trace_by_event_id.unwrap();
    println!("📊 Captured span attributes:");
    for (key, value) in &attributes {
        println!("  {}: {}", key, value);
    }
    
    assert_eq!(attributes.get("gcp.vertex.agent.invocation_id"), Some(&"test-inv-123".to_string()));
    assert_eq!(attributes.get("gcp.vertex.agent.session_id"), Some(&"test-session-456".to_string()));
    assert_eq!(attributes.get("span_name"), Some(&"agent.execute".to_string()));
    
    // Verify session trace retrieval
    let session_traces = exporter.get_session_trace("test-session-456");
    assert_eq!(session_traces.len(), 1, "Should find one span for session");
    
    println!("📊 Session trace (first span):");
    if let Some(first_span) = session_traces.first() {
        for (key, value) in first_span {
            println!("  {}: {}", key, value);
        }
    }
    
    println!("✅ ADK-Go style tracing test passed!");
}

#[tokio::test]
async fn test_span_filtering() {
    // Use the same global exporter from the first test
    // (since telemetry can only be initialized once)
    let exporter = init_with_adk_exporter("test-service-2").expect("Failed to init telemetry");
    
    // This should be ignored (not in allowed span names)
    let ignored_span = info_span!(
        "some.random.span",
        "gcp.vertex.agent.event_id" = "ignored-event"
    );
    
    async {
        tracing::info!("This should be ignored");
    }
    .instrument(ignored_span)
    .await;
    
    // This should be captured (agent.execute is allowed)
    let captured_span = info_span!(
        "agent.execute", 
        "gcp.vertex.agent.event_id" = "captured-event"
    );
    
    async {
        tracing::info!("This should be captured");
    }
    .instrument(captured_span)
    .await;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    // Debug: print all trace dict contents
    let trace_dict = exporter.get_trace_dict();
    println!("All traces: {:?}", trace_dict.keys().collect::<Vec<_>>());
    
    // Verify filtering worked
    assert!(exporter.get_trace_by_event_id("ignored-event").is_none(), "Should ignore non-allowed spans");
    assert!(exporter.get_trace_by_event_id("captured-event").is_some(), "Should capture allowed spans");
    
    println!("✅ Span filtering test passed!");
}
