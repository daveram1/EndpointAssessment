use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use chrono::Utc;
use common::{
    AgentCheckDefinition, CheckStatus, ChecksResponse, EndpointStatus,
    HeartbeatRequest, HeartbeatResponse, RegisterRequest, RegisterResponse, Severity,
    SubmitResultsRequest, SubmitResultsResponse,
};

use crate::api::ApiError;
use crate::AppState;
use crate::db::{checks, endpoints, results, snapshots};

const AGENT_SECRET_HEADER: &str = "x-agent-secret";

fn verify_agent_secret(headers: &HeaderMap, expected_secret: &str) -> Result<(), ApiError> {
    let provided = headers
        .get(AGENT_SECRET_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("Missing agent secret header"))?;

    if provided != expected_secret {
        return Err(ApiError::unauthorized("Invalid agent secret"));
    }

    Ok(())
}

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
    verify_agent_secret(&headers, &state.config.agent_secret)?;

    tracing::info!("Agent registration request from hostname: {}", req.hostname);

    let endpoint = endpoints::create_endpoint(
        &state.pool,
        &req.hostname,
        &req.os,
        &req.os_version,
        &req.agent_version,
        &req.ip_addresses,
    )
    .await?;

    Ok(Json(RegisterResponse {
        endpoint_id: endpoint.id,
        message: "Registration successful".to_string(),
    }))
}

pub async fn heartbeat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<HeartbeatRequest>,
) -> Result<Json<HeartbeatResponse>, ApiError> {
    verify_agent_secret(&headers, &state.config.agent_secret)?;

    // Verify endpoint exists
    let endpoint = endpoints::get_endpoint_by_id(&state.pool, req.endpoint_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Endpoint not found"))?;

    tracing::debug!("Heartbeat from endpoint: {} ({})", endpoint.hostname, endpoint.id);

    // Store snapshot
    snapshots::create_snapshot(
        &state.pool,
        req.endpoint_id,
        req.snapshot.cpu_usage,
        req.snapshot.memory_total as i64,
        req.snapshot.memory_used as i64,
        req.snapshot.disk_total as i64,
        req.snapshot.disk_used as i64,
        &req.snapshot.processes,
        &req.snapshot.open_ports,
        &req.snapshot.installed_software,
        req.snapshot.collected_at,
    )
    .await?;

    // Update endpoint status
    endpoints::update_endpoint_heartbeat(&state.pool, req.endpoint_id, EndpointStatus::Online).await?;

    Ok(Json(HeartbeatResponse {
        status: "ok".to_string(),
        server_time: Utc::now(),
    }))
}

pub async fn get_checks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ChecksResponse>, ApiError> {
    verify_agent_secret(&headers, &state.config.agent_secret)?;

    let check_rows = checks::list_enabled_checks(&state.pool).await?;

    let checks: Vec<AgentCheckDefinition> = check_rows
        .into_iter()
        .map(|row| AgentCheckDefinition {
            id: row.id,
            name: row.name,
            check_type: row.check_type,
            parameters: row.parameters,
            severity: row.severity.parse().unwrap_or(Severity::Medium),
        })
        .collect();

    Ok(Json(ChecksResponse { checks }))
}

pub async fn submit_results(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SubmitResultsRequest>,
) -> Result<Json<SubmitResultsResponse>, ApiError> {
    verify_agent_secret(&headers, &state.config.agent_secret)?;

    // Verify endpoint exists
    let endpoint = endpoints::get_endpoint_by_id(&state.pool, req.endpoint_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Endpoint not found"))?;

    tracing::debug!(
        "Receiving {} check results from endpoint: {} ({})",
        req.results.len(),
        endpoint.hostname,
        endpoint.id
    );

    let mut accepted = 0;
    let mut has_failures = false;

    for result in &req.results {
        match results::create_result(
            &state.pool,
            req.endpoint_id,
            result.check_id,
            result.status,
            result.message.as_deref(),
            result.collected_at,
        )
        .await
        {
            Ok(_) => {
                accepted += 1;
                if result.status == CheckStatus::Fail {
                    has_failures = true;
                }
            }
            Err(e) => {
                tracing::warn!("Failed to store check result: {:?}", e);
            }
        }
    }

    // Update endpoint status based on results
    let new_status = if has_failures {
        EndpointStatus::Warning
    } else {
        EndpointStatus::Online
    };
    endpoints::update_endpoint_heartbeat(&state.pool, req.endpoint_id, new_status).await?;

    Ok(Json(SubmitResultsResponse {
        accepted,
        message: format!("Accepted {} results", accepted),
    }))
}
