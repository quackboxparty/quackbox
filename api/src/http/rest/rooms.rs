use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};

use crate::{
    game::room::{JoinCode, spawn_room},
    state::AppState,
};

#[derive(Serialize, Deserialize)]
struct CreateRoom {
    secret: String,
}

#[derive(Serialize, Deserialize)]
struct Room {
    join_code: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/rooms", post(create_room))
}

async fn create_room(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateRoom>,
) -> impl IntoResponse {
    const MAX_TRIES: usize = 10;

    let Some(code) = (0..MAX_TRIES)
        .map(|_| JoinCode::generate())
        .find(|code| !state.rooms.contains_key(code))
    else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "no free join code").into_response();
    };

    let handle = spawn_room(code.clone());
    state.rooms.insert(code.clone(), handle);

    Json(Room { join_code: code.0 }).into_response()
}
