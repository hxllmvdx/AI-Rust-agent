use api::{health, sessions};
use axum::{
    Router,
    routing::{get, post},
};
use state::AppState;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::services::session_store::SessionStore;

pub mod api;
pub mod config;
pub mod error;
pub mod models;
pub mod services;
pub mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    let config = config::Config::from_env()?;

    let redis_client = redis::Client::open(config.redis_url.clone())?;
    let session_store = SessionStore::new(redis_client, config.session_ttl);

    let state = AppState {
        app_name: "ai-rust-agent".to_string(),
        sessions: session_store,
    };

    let app = Router::new()
        .route("/health", get(health::health_handler))
        .route("/sessions", post(sessions::create_session_handler))
        .route(
            "/history/{session_id}",
            get(sessions::create_session_handler),
        )
        .route("/reset/{session_id}", post(sessions::reset_session_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
