use api::health;
use axum::{Router, routing::get};
use state::AppState;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub mod api;
pub mod config;
pub mod error;
pub mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    let config = config::Config::from_env()?;

    let state = AppState {
        app_name: "ai-rust-agent".to_string(),
    };

    let app = Router::new()
        .route("/health", get(health::health_handler))
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
