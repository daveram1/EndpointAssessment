mod api;
mod config;
mod db;
mod services;
mod web;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load .env if present
    let _ = dotenvy::dotenv();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    let addr = config.socket_addr();

    tracing::info!("Connecting to database...");
    let pool = db::create_pool(&config.database_url).await?;

    tracing::info!("Running database migrations...");
    sqlx::migrate!("../migrations").run(&pool).await?;

    let state = AppState {
        pool: pool.clone(),
        config: Arc::new(config.clone()),
    };

    // Start background tasks
    services::start_background_tasks(pool, config.offline_threshold_minutes).await;

    // Build router
    let app = Router::new()
        // Agent API routes
        .route("/api/agent/register", post(api::agent::register))
        .route("/api/agent/heartbeat", post(api::agent::heartbeat))
        .route("/api/agent/checks", get(api::agent::get_checks))
        .route("/api/agent/results", post(api::agent::submit_results))
        // Admin API routes
        .route("/api/endpoints", get(api::admin::list_endpoints))
        .route("/api/endpoints/:id", get(api::admin::get_endpoint))
        .route("/api/endpoints/:id", delete(api::admin::delete_endpoint))
        .route("/api/checks", get(api::admin::list_checks))
        .route("/api/checks", post(api::admin::create_check))
        .route("/api/checks/:id", get(api::admin::get_check))
        .route("/api/checks/:id", put(api::admin::update_check))
        .route("/api/checks/:id", delete(api::admin::delete_check))
        .route("/api/results", get(api::admin::list_results))
        .route("/api/reports/summary", get(api::admin::get_summary))
        // Web UI routes
        .route("/", get(web::routes::dashboard))
        .route("/endpoints", get(web::routes::endpoints_list))
        .route("/endpoints/:id", get(web::routes::endpoint_detail))
        .route("/endpoints/:id/delete", post(web::routes::endpoint_delete))
        .route("/checks", get(web::routes::checks_list))
        .route("/checks/new", get(web::routes::check_new))
        .route("/checks", post(web::routes::check_create))
        .route("/checks/:id/edit", get(web::routes::check_edit))
        .route("/checks/:id", post(web::routes::check_update))
        .route("/checks/:id/delete", post(web::routes::check_delete))
        .route("/reports", get(web::routes::reports))
        // Auth routes
        .route("/login", get(web::routes::login_page))
        .route("/login", post(web::routes::login_submit))
        .route("/logout", get(web::routes::logout))
        .route("/setup", get(web::routes::setup_page))
        .route("/setup", post(web::routes::setup_submit))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
