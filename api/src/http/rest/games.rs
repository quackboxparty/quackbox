use std::sync::Arc;

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Games.ts"))]
struct Game {
    id: String,
    title: String,
    description: String,
    modes: Vec<String>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/games", get(list_games))
}

async fn list_games(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let games = &state.data.games;
    Json(
        games
            .iter()
            .map(|(id, game)| Game {
                id: id.clone(),
                title: game.item.title.clone(),
                description: game.item.description.clone(),
                modes: game
                    .item
                    .games
                    .iter()
                    .map(|mode| mode.mode_name().to_owned())
                    .collect(),
            })
            .collect::<Vec<Game>>(),
    )
}

