//! `adk-3d-ui` provides a minimal agentic 3D UI runtime for ADK-Rust.
//! It uses SSE for server-to-client updates and HTTP for client events.

pub mod executor;
pub mod planner;
pub mod policy;
pub mod protocol;
pub mod server;
pub mod session;

pub use server::{AppState, ServerConfig, app_router, run_server};
