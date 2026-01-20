use axum::{
    extract::{Path, State, Query},
    Json,
};
use common::{CheckStatus, DashboardSummary, Endpoint, RecentCheckResult, Severity, SystemSnapshot};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::ApiError;
use crate::AppState;
use crate::db::{checks, endpoints, results, snapshots};

// Endpoints

pub async fn list_endpoints(
    State(state): State<AppState>,
) -> Result<Json<Vec<Endpoint>>, ApiError> {
    let endpoints = endpoints::list_endpoints(&state.pool).await?;
    Ok(Json(endpoints))
}

pub async fn get_endpoint(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<EndpointDetail>, ApiError> {
    let endpoint = endpoints::get_endpoint_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::not_found("Endpoint not found"))?;

    let latest_results = results::get_latest_results_for_endpoint(&state.pool, id).await?;
    let latest_snapshot = snapshots::get_latest_snapshot(&state.pool, id).await?;

    let check_results: Vec<EndpointCheckResult> = latest_results
        .into_iter()
        .map(|r| EndpointCheckResult {
            check_id: r.check_id,
            check_name: r.check_name,
            status: r.status.parse().unwrap_or(CheckStatus::Error),
            message: r.message,
            collected_at: r.collected_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(EndpointDetail {
        endpoint,
        latest_snapshot,
        check_results,
    }))
}

#[derive(Debug, Serialize)]
pub struct EndpointDetail {
    pub endpoint: Endpoint,
    pub latest_snapshot: Option<SystemSnapshot>,
    pub check_results: Vec<EndpointCheckResult>,
}

#[derive(Debug, Serialize)]
pub struct EndpointCheckResult {
    pub check_id: Uuid,
    pub check_name: String,
    pub status: CheckStatus,
    pub message: Option<String>,
    pub collected_at: String,
}

pub async fn delete_endpoint(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteResponse>, ApiError> {
    let deleted = endpoints::delete_endpoint(&state.pool, id).await?;

    if deleted {
        Ok(Json(DeleteResponse {
            success: true,
            message: "Endpoint deleted".to_string(),
        }))
    } else {
        Err(ApiError::not_found("Endpoint not found"))
    }
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub message: String,
}

// Checks

pub async fn list_checks(
    State(state): State<AppState>,
) -> Result<Json<Vec<CheckDefinitionResponse>>, ApiError> {
    let check_list = checks::list_checks(&state.pool).await?;

    let response: Vec<CheckDefinitionResponse> = check_list
        .into_iter()
        .map(|c| CheckDefinitionResponse {
            id: c.id,
            name: c.name,
            description: c.description,
            check_type: c.check_type,
            parameters: c.parameters,
            severity: c.severity.parse().unwrap_or(Severity::Medium),
            enabled: c.enabled,
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

#[derive(Debug, Serialize)]
pub struct CheckDefinitionResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub check_type: String,
    pub parameters: serde_json::Value,
    pub severity: Severity,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn get_check(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CheckDefinitionResponse>, ApiError> {
    let check = checks::get_check_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::not_found("Check not found"))?;

    Ok(Json(CheckDefinitionResponse {
        id: check.id,
        name: check.name,
        description: check.description,
        check_type: check.check_type,
        parameters: check.parameters,
        severity: check.severity.parse().unwrap_or(Severity::Medium),
        enabled: check.enabled,
        created_at: check.created_at.to_rfc3339(),
        updated_at: check.updated_at.to_rfc3339(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct CreateCheckRequest {
    pub name: String,
    pub description: Option<String>,
    pub check_type: String,
    pub parameters: serde_json::Value,
    #[serde(default)]
    pub severity: Option<Severity>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

pub async fn create_check(
    State(state): State<AppState>,
    Json(req): Json<CreateCheckRequest>,
) -> Result<Json<CheckDefinitionResponse>, ApiError> {
    let severity = req.severity.unwrap_or(Severity::Medium);

    let check = checks::create_check(
        &state.pool,
        &req.name,
        req.description.as_deref(),
        &req.check_type,
        req.parameters,
        severity,
        req.enabled,
    )
    .await?;

    Ok(Json(CheckDefinitionResponse {
        id: check.id,
        name: check.name,
        description: check.description,
        check_type: check.check_type,
        parameters: check.parameters,
        severity: check.severity.parse().unwrap_or(Severity::Medium),
        enabled: check.enabled,
        created_at: check.created_at.to_rfc3339(),
        updated_at: check.updated_at.to_rfc3339(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateCheckRequest {
    pub name: String,
    pub description: Option<String>,
    pub check_type: String,
    pub parameters: serde_json::Value,
    pub severity: Severity,
    pub enabled: bool,
}

pub async fn update_check(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCheckRequest>,
) -> Result<Json<CheckDefinitionResponse>, ApiError> {
    let check = checks::update_check(
        &state.pool,
        id,
        &req.name,
        req.description.as_deref(),
        &req.check_type,
        req.parameters,
        req.severity,
        req.enabled,
    )
    .await?
    .ok_or_else(|| ApiError::not_found("Check not found"))?;

    Ok(Json(CheckDefinitionResponse {
        id: check.id,
        name: check.name,
        description: check.description,
        check_type: check.check_type,
        parameters: check.parameters,
        severity: check.severity.parse().unwrap_or(Severity::Medium),
        enabled: check.enabled,
        created_at: check.created_at.to_rfc3339(),
        updated_at: check.updated_at.to_rfc3339(),
    }))
}

pub async fn delete_check(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteResponse>, ApiError> {
    let deleted = checks::delete_check(&state.pool, id).await?;

    if deleted {
        Ok(Json(DeleteResponse {
            success: true,
            message: "Check deleted".to_string(),
        }))
    } else {
        Err(ApiError::not_found("Check not found"))
    }
}

// Results

#[derive(Debug, Deserialize)]
pub struct ResultsQuery {
    pub endpoint_id: Option<Uuid>,
    pub check_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    100
}

pub async fn list_results(
    State(state): State<AppState>,
    Query(query): Query<ResultsQuery>,
) -> Result<Json<Vec<ResultResponse>>, ApiError> {
    let result_rows = if let Some(endpoint_id) = query.endpoint_id {
        results::get_results_for_endpoint(&state.pool, endpoint_id, query.limit).await?
    } else if let Some(check_id) = query.check_id {
        results::get_results_for_check(&state.pool, check_id, query.limit).await?
    } else {
        // Return recent results
        let recent = results::get_recent_results(&state.pool, query.limit).await?;
        return Ok(Json(
            recent
                .into_iter()
                .map(|r| ResultResponse {
                    id: r.id,
                    endpoint_id: None,
                    endpoint_hostname: Some(r.endpoint_hostname),
                    check_id: None,
                    check_name: Some(r.check_name),
                    status: r.status.parse().unwrap_or(CheckStatus::Error),
                    message: r.message,
                    collected_at: r.collected_at.to_rfc3339(),
                })
                .collect(),
        ));
    };

    let response: Vec<ResultResponse> = result_rows
        .into_iter()
        .map(|r| ResultResponse {
            id: r.id,
            endpoint_id: Some(r.endpoint_id),
            endpoint_hostname: None,
            check_id: Some(r.check_id),
            check_name: None,
            status: r.status.parse().unwrap_or(CheckStatus::Error),
            message: r.message,
            collected_at: r.collected_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

#[derive(Debug, Serialize)]
pub struct ResultResponse {
    pub id: Uuid,
    pub endpoint_id: Option<Uuid>,
    pub endpoint_hostname: Option<String>,
    pub check_id: Option<Uuid>,
    pub check_name: Option<String>,
    pub status: CheckStatus,
    pub message: Option<String>,
    pub collected_at: String,
}

// Dashboard summary

pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<DashboardSummary>, ApiError> {
    let endpoint_counts = endpoints::get_endpoint_counts(&state.pool).await?;
    let check_counts = checks::get_check_counts(&state.pool).await?;
    let recent = results::get_recent_results(&state.pool, 10).await?;

    let recent_results: Vec<RecentCheckResult> = recent
        .into_iter()
        .map(|r| RecentCheckResult {
            endpoint_hostname: r.endpoint_hostname,
            check_name: r.check_name,
            status: r.status.parse().unwrap_or(CheckStatus::Error),
            message: r.message,
            collected_at: r.collected_at,
        })
        .collect();

    Ok(Json(DashboardSummary {
        total_endpoints: endpoint_counts.total,
        online_endpoints: endpoint_counts.online,
        offline_endpoints: endpoint_counts.offline,
        warning_endpoints: endpoint_counts.warning,
        critical_endpoints: endpoint_counts.critical,
        total_checks: check_counts.total,
        enabled_checks: check_counts.enabled,
        recent_results,
    }))
}
