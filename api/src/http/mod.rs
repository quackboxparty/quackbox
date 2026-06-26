//! Transport layer — everything axum-aware. Routes, extractors, JSON, the WS
//! upgrade. This module is the ONLY place that imports `axum`.
//!
//! `build_router(state)` assembles the full `Router`: nests the REST sub-router
//! (`rest`), the WebSocket route (`ws`), the static `build/` fallback, and the
//! `/api/health` ops route. `main.rs` calls this and serves the result.

pub mod rest;
pub mod ws;

// TODO: pub fn build_router(state: Arc<AppState>) -> Router
