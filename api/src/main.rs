mod config;
use std::sync::Arc;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{AppConfig, load};

struct AppState {
    config: AppConfig,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        config: load().expect("couldn't load config"),
    });

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let serve_dir =
        ServeDir::new("../build").not_found_service(ServeFile::new("../build/index.html"));

    let app = Router::new()
        .fallback_service(serve_dir)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", state.config.host, state.config.port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
