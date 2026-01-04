use std::sync::Arc;
use std::time::Duration;
use tracing::info_span;
use crate::memory::SharedTraceStorage;

#[tokio::test]
async fn test_inmemory_tracing_capture() {
    // Create trace storage
    let storage = Arc::new(SharedTraceStorage::new());
    
    // Initialize telemetry with our storage - this sets up the global subscriber
    println!("Initializing telemetry with our storage...");
    let result = crate::init_with_storage("test-service", storage.clone());
    println!("Init result: {:?}", result);
    
    // Create a span with dot notation like the agent uses
    let span = info_span!(
        "agent.execute",
        invocation.id = "test-inv-123",
        session.id = "test-session-456", 
        agent.name = "test-agent"
    );
    
    // Execute some work within the span
    {
        let _guard = span.enter();
        tracing::info!("Test span execution");
        // _guard is dropped here, closing the span
    }
    
    // Give telemetry time to process
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Check if spans were captured in our storage instance
    let traces = storage.get_trace("test-inv-123");
    assert!(traces.is_some(), "Should capture spans under invocation_id");
    
    let spans = traces.unwrap();
    assert!(!spans.is_empty(), "Should have captured at least one span");
    
    let span = &spans[0];
    assert_eq!(span.name, "agent.execute");
    assert!(span.attributes.contains_key("invocation.id"));
    assert!(span.start_time > 0);
    assert!(span.end_time.is_some());
}

#[tokio::test] 
async fn test_session_id_storage() {
    let storage = Arc::new(SharedTraceStorage::new());
    let _ = crate::init_with_storage("test-service", storage.clone());
    
    let span = info_span!(
        "test.span",
        session.id = "session-789"
    );
    
    let _guard = span.enter();
    tracing::info!("Session span test");
    drop(_guard);
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Should be stored under session_id
    let traces = storage.get_trace("session-789");
    assert!(traces.is_some(), "Should capture spans under session_id");
}
