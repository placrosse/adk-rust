#![cfg(feature = "vertex-session")]

mod common;

use adk_session::{VertexAiSessionConfig, VertexAiSessionService};
use uuid::Uuid;

const ENV_PROJECT_ID_KEYS: [&str; 2] = ["GOOGLE_PROJECT_ID", "GOOGLE_CLOUD_PROJECT"];
const ENV_LOCATION_KEYS: [&str; 2] = ["GOOGLE_CLOUD_LOCATION", "GOOGLE_VERTEX_LOCATION"];
const ENV_APP_NAME_KEYS: [&str; 2] = ["GOOGLE_VERTEX_APP_NAME", "ADK_VERTEX_SESSION_APP_NAME"];
const ENV_OTHER_APP_NAME_KEYS: [&str; 2] =
    ["GOOGLE_VERTEX_OTHER_APP_NAME", "ADK_VERTEX_SESSION_OTHER_APP_NAME"];

fn required_env_any(keys: &[&str]) -> String {
    for key in keys {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    panic!("one of [{}] is required for live Vertex session contract test", keys.join(", "))
}

#[tokio::test]
#[ignore = "requires live Vertex Session Service resources + ADC; run with --ignored"]
async fn test_vertex_service_live_contract() {
    let project_id = required_env_any(&ENV_PROJECT_ID_KEYS);
    let location = required_env_any(&ENV_LOCATION_KEYS);
    let app_name = required_env_any(&ENV_APP_NAME_KEYS);
    let other_app_name = required_env_any(&ENV_OTHER_APP_NAME_KEYS);

    let service =
        VertexAiSessionService::new_with_adc(VertexAiSessionConfig::new(project_id, location))
            .expect("build vertex session service");

    let run_id = Uuid::new_v4().simple().to_string();
    let user_1 = format!("adk-rust-live-u1-{run_id}");
    let user_2 = format!("adk-rust-live-u2-{run_id}");

    common::session_contract::assert_session_contract_with_users(
        &service,
        &app_name,
        &other_app_name,
        &user_1,
        &user_2,
    )
    .await;
}
