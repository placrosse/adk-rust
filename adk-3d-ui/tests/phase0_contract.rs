use std::time::Duration;

use adk_3d_ui::{app_router, protocol::UiEventAck, server::AppState};
use serde_json::Value;

async fn spawn_server() -> (String, tokio::task::JoinHandle<()>) {
    let state = AppState::default();
    let app = app_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let addr = listener.local_addr().expect("listener addr");

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("server run");
    });

    (format!("http://{}", addr), handle)
}

#[tokio::test]
async fn phase0_create_session_and_post_select_event() {
    let (base, handle) = spawn_server().await;
    let client = reqwest::Client::new();

    let create = client
        .post(format!("{}/api/3d/session", base))
        .send()
        .await
        .expect("session create response");
    assert!(create.status().is_success());

    let created: Value = create.json().await.expect("session json");
    let session_id = created
        .get("session_id")
        .and_then(Value::as_str)
        .expect("session_id field");

    let ack = client
        .post(format!("{}/api/3d/event/{}", base, session_id))
        .json(&serde_json::json!({
            "seq": 1,
            "event": {
                "type": "select",
                "id": "service-payments"
            }
        }))
        .send()
        .await
        .expect("event post response");

    assert!(ack.status().is_success());
    let body: UiEventAck = ack.json().await.expect("ack json");
    assert!(body.ok);

    handle.abort();
}

#[tokio::test]
async fn phase0_stream_endpoint_is_sse() {
    let (base, handle) = spawn_server().await;
    let client = reqwest::Client::new();

    let create = client
        .post(format!("{}/api/3d/session", base))
        .send()
        .await
        .expect("session create response");
    let created: Value = create.json().await.expect("session json");
    let session_id = created
        .get("session_id")
        .and_then(Value::as_str)
        .expect("session_id field");

    let response = tokio::time::timeout(
        Duration::from_secs(3),
        client.get(format!("{}/api/3d/stream/{}", base, session_id)).send(),
    )
    .await
    .expect("stream request timeout")
    .expect("stream response");

    assert!(response.status().is_success());
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_lowercase();
    assert!(
        content_type.contains("text/event-stream"),
        "unexpected content type: {content_type}"
    );

    handle.abort();
}
