use adk_server::create_app;
use adk_session::{
    CreateRequest, DeleteRequest, Event, GetRequest, ListRequest, Session, SessionService,
};
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tower::ServiceExt;

struct MockAgentLoader;

#[async_trait]
impl adk_core::AgentLoader for MockAgentLoader {
    async fn load_agent(&self, _app_name: &str) -> adk_core::Result<Arc<dyn adk_core::Agent>> {
        Err(adk_core::AdkError::Agent("not implemented".to_string()))
    }

    fn list_agents(&self) -> Vec<String> {
        vec![]
    }

    fn root_agent(&self) -> Arc<dyn adk_core::Agent> {
        panic!("MockAgentLoader has no root agent")
    }
}

struct MockSessionService;

#[async_trait]
impl SessionService for MockSessionService {
    async fn create(&self, req: CreateRequest) -> adk_core::Result<Box<dyn Session>> {
        Ok(Box::new(MockSession {
            id: req.session_id.unwrap_or_else(|| "generated-id".to_string()),
            app_name: req.app_name,
            user_id: req.user_id,
        }))
    }

    async fn get(&self, req: GetRequest) -> adk_core::Result<Box<dyn Session>> {
        Ok(Box::new(MockSession {
            id: req.session_id,
            app_name: req.app_name,
            user_id: req.user_id,
        }))
    }

    async fn list(&self, _req: ListRequest) -> adk_core::Result<Vec<Box<dyn Session>>> {
        Ok(vec![])
    }

    async fn delete(&self, _req: DeleteRequest) -> adk_core::Result<()> {
        Ok(())
    }

    async fn append_event(&self, _session_id: &str, _event: Event) -> adk_core::Result<()> {
        Ok(())
    }
}

struct MockSession {
    id: String,
    app_name: String,
    user_id: String,
}

impl Session for MockSession {
    fn id(&self) -> &str {
        &self.id
    }

    fn app_name(&self) -> &str {
        &self.app_name
    }

    fn user_id(&self) -> &str {
        &self.user_id
    }

    fn state(&self) -> &dyn adk_session::State {
        &MockState
    }

    fn events(&self) -> &dyn adk_session::Events {
        &MockEvents
    }

    fn last_update_time(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

struct MockState;
impl adk_session::State for MockState {
    fn get(&self, _key: &str) -> Option<serde_json::Value> {
        None
    }
    fn set(&mut self, _key: String, _value: serde_json::Value) {}
    fn all(&self) -> std::collections::HashMap<String, serde_json::Value> {
        std::collections::HashMap::new()
    }
}

struct MockEvents;
impl adk_session::Events for MockEvents {
    fn all(&self) -> Vec<Event> {
        vec![]
    }
    fn len(&self) -> usize {
        0
    }
    fn at(&self, _index: usize) -> Option<&Event> {
        None
    }
}

#[tokio::test]
async fn test_health_check() {
    let config =
        adk_server::ServerConfig::new(Arc::new(MockAgentLoader), Arc::new(MockSessionService));
    let app = create_app(config);

    let response = app
        .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ui_capabilities() {
    let config =
        adk_server::ServerConfig::new(Arc::new(MockAgentLoader), Arc::new(MockSessionService));
    let app = create_app(config);

    let response = app
        .oneshot(
            Request::builder().uri("/api/ui/capabilities").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["default_protocol"], "adk_ui");
    assert!(json["protocols"].as_array().unwrap().iter().any(|entry| entry["protocol"] == "a2ui"));
    assert!(
        json["protocols"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["protocol"] == "ag_ui")
    );
    assert!(
        json["protocols"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["protocol"] == "mcp_apps")
    );
}

#[tokio::test]
async fn test_run_sse_compat_rejects_unknown_ui_protocol() {
    let config =
        adk_server::ServerConfig::new(Arc::new(MockAgentLoader), Arc::new(MockSessionService));
    let app = create_app(config);

    let body = serde_json::json!({
        "appName": "test-app",
        "userId": "user1",
        "sessionId": "session1",
        "newMessage": {
            "role": "user",
            "parts": [{ "text": "hello" }]
        },
        "uiProtocol": "unknown_protocol"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/run_sse")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("Supported profiles"));
    assert!(body_str.contains("adk_ui"));
    assert!(body_str.contains("a2ui"));
}

#[tokio::test]
async fn test_run_sse_compat_header_precedence_over_body_protocol() {
    let config =
        adk_server::ServerConfig::new(Arc::new(MockAgentLoader), Arc::new(MockSessionService));
    let app = create_app(config);

    let body = serde_json::json!({
        "appName": "test-app",
        "userId": "user1",
        "sessionId": "session1",
        "newMessage": {
            "role": "user",
            "parts": [{ "text": "hello" }]
        },
        "uiProtocol": "unknown_protocol"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/run_sse")
                .header("content-type", "application/json")
                .header("x-adk-ui-protocol", "a2ui")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // The header protocol takes precedence, so request proceeds past protocol validation.
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_run_path_rejects_unknown_ui_protocol_header() {
    let config =
        adk_server::ServerConfig::new(Arc::new(MockAgentLoader), Arc::new(MockSessionService));
    let app = create_app(config);

    let body = serde_json::json!({
        "new_message": "hello"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/run/test-app/user1/session1")
                .header("content-type", "application/json")
                .header("x-adk-ui-protocol", "bad-profile")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
