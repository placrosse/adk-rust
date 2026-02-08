//! ADK Spatial OS: AI-native shell where apps are ADK-Rust agents.

pub mod app_runtime;
pub mod protocol;
pub mod safety;
pub mod server;
pub mod session;
pub mod shell;

pub use server::{AppState, ServerConfig, app_router, run_server};
