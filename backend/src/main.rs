use crate::{agent::planner::PlannerService, tools::local_data::LocalKnowledgeTool};
use api::{debug_llm, debug_local_search, debug_plan, health, sessions};
use axum::{
    Router,
    routing::{get, post},
};
use state::AppState;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::services::{llm::LlmService, session_store::SessionStore};

pub mod agent;
pub mod api;
pub mod config;
pub mod error;
pub mod models;
pub mod services;
pub mod state;
pub mod tools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env()?;

    let redis_client = redis::Client::open(config.redis_url.clone())?;
    let session_store = SessionStore::new(redis_client, config.session_ttl);

    let llm = LlmService::new(config.ollama_url.clone(), config.ollama_model.clone());
    let planner = PlannerService::new(llm.clone());

    let local_tool = LocalKnowledgeTool::load_from_file("/app/data/rust_tools.json")?;

    let state = AppState {
        app_name: "ai-rust-agent".to_string(),
        sessions: session_store,
        llm: llm,
        planner: planner,
        local_tool: local_tool,
    };

    let app = Router::new()
        .route("/health", get(health::health_handler))
        .route("/sessions", post(sessions::create_session_handler))
        .route(
            "/history/{session_id}",
            get(sessions::create_session_handler),
        )
        .route("/reset/{session_id}", post(sessions::reset_session_handler))
        .route("/debug/llm", post(debug_llm::debug_llm_handler))
        .route("/debug/plan", post(debug_plan::debug_plan_handler))
        .route(
            "/debug/local-search",
            post(debug_local_search::debug_local_search_handler),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
