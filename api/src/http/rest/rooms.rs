use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    game::room::{JoinCode, spawn_room},
    state::AppState,
};

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Rooms.ts"))]
struct CreateRoom {
    game_id: String,
    secret: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Rooms.ts"))]
struct Room {
    join_code: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/rooms", post(create_room))
        .route("/rooms/{code}", get(check_room))
}

async fn create_room(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateRoom>,
) -> impl IntoResponse {
    if let Some(secret) = state.config.admin_secret.as_deref()
        && body.secret.as_deref() != Some(secret)
    {
        return StatusCode::FORBIDDEN.into_response();
    }

    const MAX_TRIES: usize = 10;

    let Some(code) = (0..MAX_TRIES)
        .map(|_| JoinCode::generate())
        .find(|code| !state.rooms.contains_key(code))
    else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "no free join code").into_response();
    };

    let Some(game) = state.data.games.get(&body.game_id) else {
        return (StatusCode::BAD_REQUEST, "game does not exist").into_response();
    };

    let handle = spawn_room(code.clone(), game.item.clone());
    state.rooms.insert(code.clone(), handle);

    Json(Room { join_code: code.0 }).into_response()
}

async fn check_room(Path(code): Path<String>, State(state): State<Arc<AppState>>) -> StatusCode {
    let code = JoinCode(code.clone());
    if state.rooms.contains_key(&code) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
