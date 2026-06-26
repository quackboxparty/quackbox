//! REST sub-router — cold, cacheable content reads under `/api/*`. One file per
//! resource. `router()` merges the per-resource routers.

use std::sync::Arc;

use axum::{Router, extract::State, routing::get};

use crate::state::AppState;

pub mod boards;
pub mod packs;
pub mod rooms;
pub mod stats;

// TODO: pub fn router(state: Arc<AppState>) -> Router  (merge boards/packs/stats)

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))
        .merge(rooms::router())
}

async fn health_check(State(state): State<Arc<AppState>>) -> String {
    format!(
        "ok, with dataset: {} questions, {} packs, {} tags and {} open rooms",
        state.data.questions.len(),
        state.data.packs.len(),
        state.data.tags.len(),
        state.rooms.len()
    )
}
