use crate::server::{handlers, sse, state::AppState};
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/projects", get(handlers::list_projects))
        .route("/projects", post(handlers::create_project))
        .route("/projects/:id", get(handlers::get_project))
        .route("/projects/:id", put(handlers::update_project))
        .route("/projects/:id", delete(handlers::delete_project))
        .route("/projects/:id/run", post(handlers::run_project))
        .route("/projects/:id/stream", get(sse::stream_handler))
}
