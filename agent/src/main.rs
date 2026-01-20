mod checks;
mod client;
mod collectors;
mod config;

use std::time::Duration;

use chrono::Utc;
use common::{AgentCheckResult, RegisterRequest};
use tokio::time::interval;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::checks::CheckExecutor;
use crate::client::ServerClient;
use crate::collectors::SystemCollector;
use crate::config::Config;

const AGENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load .env if present
    let _ = dotenvy::dotenv();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let config = if args.len() >= 3 {
        // Command line: agent <server_url> <agent_secret>
        let server_url = args.get(1).map(|s| s.as_str()).unwrap_or("http://localhost:8080");
        let agent_secret = args.get(2).map(|s| s.as_str()).unwrap_or("change-me-in-production");
        Config::from_args(server_url.to_string(), agent_secret.to_string())
    } else {
        Config::from_env().expect("Failed to load configuration. Set SERVER_URL and AGENT_SECRET environment variables, or pass them as arguments.")
    };

    tracing::info!("Starting Endpoint Assessment Agent v{}", AGENT_VERSION);
    tracing::info!("Server URL: {}", config.server_url);

    // Initialize components
    let mut collector = SystemCollector::new();
    let mut executor = CheckExecutor::new();
    let client = ServerClient::new(&config.server_url, &config.agent_secret);

    // Register with server
    let hostname = config
        .hostname_override
        .clone()
        .unwrap_or_else(|| collector.get_hostname());

    tracing::info!("Registering endpoint: {}", hostname);

    let register_request = RegisterRequest {
        hostname: hostname.clone(),
        os: collector.get_os(),
        os_version: collector.get_os_version(),
        agent_version: AGENT_VERSION.to_string(),
        ip_addresses: collector.get_ip_addresses(),
    };

    let endpoint_id = loop {
        match client.register(register_request.clone()).await {
            Ok(response) => {
                tracing::info!("Registered successfully. Endpoint ID: {}", response.endpoint_id);
                break response.endpoint_id;
            }
            Err(e) => {
                tracing::error!("Registration failed: {}. Retrying in 30 seconds...", e);
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        }
    };

    // Main collection loop
    let mut ticker = interval(Duration::from_secs(config.collection_interval_secs));

    tracing::info!(
        "Starting collection loop (interval: {} seconds)",
        config.collection_interval_secs
    );

    loop {
        ticker.tick().await;

        tracing::debug!("Starting collection cycle");

        // Collect system snapshot
        let snapshot = collector.collect_snapshot();

        // Send heartbeat
        match client.heartbeat(endpoint_id, snapshot).await {
            Ok(_) => tracing::debug!("Heartbeat sent successfully"),
            Err(e) => tracing::error!("Failed to send heartbeat: {}", e),
        }

        // Fetch and execute checks
        match client.get_checks().await {
            Ok(checks_response) => {
                if checks_response.checks.is_empty() {
                    tracing::debug!("No checks to execute");
                    continue;
                }

                tracing::info!("Executing {} checks", checks_response.checks.len());

                let mut results: Vec<AgentCheckResult> = Vec::new();

                for check in &checks_response.checks {
                    tracing::debug!("Executing check: {} ({})", check.name, check.check_type);

                    let result = executor.execute(check);

                    tracing::info!(
                        "Check '{}': {:?} - {}",
                        check.name,
                        result.status,
                        result.message.as_deref().unwrap_or("")
                    );

                    results.push(AgentCheckResult {
                        check_id: check.id,
                        status: result.status,
                        message: result.message,
                        collected_at: Utc::now(),
                    });
                }

                // Submit results
                match client.submit_results(endpoint_id, results).await {
                    Ok(response) => {
                        tracing::info!("Submitted {} check results", response.accepted);
                    }
                    Err(e) => {
                        tracing::error!("Failed to submit check results: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to fetch checks: {}", e);
            }
        }

        tracing::debug!("Collection cycle complete");
    }
}
