use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{CheckResult, CheckStatus, ProcessInfo, Severity, SoftwareInfo, SystemSnapshot};

/// Agent registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub agent_version: String,
    pub ip_addresses: Vec<String>,
}

/// Agent registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub endpoint_id: Uuid,
    pub message: String,
}

/// Heartbeat request from agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub endpoint_id: Uuid,
    pub snapshot: SystemSnapshotData,
}

/// System snapshot data sent in heartbeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshotData {
    pub collected_at: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub processes: Vec<ProcessInfo>,
    pub open_ports: Vec<u16>,
    pub installed_software: Vec<SoftwareInfo>,
}

impl SystemSnapshotData {
    pub fn into_snapshot(self, endpoint_id: Uuid) -> SystemSnapshot {
        SystemSnapshot {
            endpoint_id,
            collected_at: self.collected_at,
            cpu_usage: self.cpu_usage,
            memory_total: self.memory_total,
            memory_used: self.memory_used,
            disk_total: self.disk_total,
            disk_used: self.disk_used,
            processes: self.processes,
            open_ports: self.open_ports,
            installed_software: self.installed_software,
        }
    }
}

/// Heartbeat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub status: String,
    pub server_time: DateTime<Utc>,
}

/// Check definition sent to agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCheckDefinition {
    pub id: Uuid,
    pub name: String,
    pub check_type: String,
    pub parameters: serde_json::Value,
    pub severity: Severity,
}

/// Response containing check definitions for agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksResponse {
    pub checks: Vec<AgentCheckDefinition>,
}

/// Single check result from agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCheckResult {
    pub check_id: Uuid,
    pub status: CheckStatus,
    pub message: Option<String>,
    pub collected_at: DateTime<Utc>,
}

/// Request to submit check results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResultsRequest {
    pub endpoint_id: Uuid,
    pub results: Vec<AgentCheckResult>,
}

/// Response after submitting results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResultsResponse {
    pub accepted: usize,
    pub message: String,
}

/// Error response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
        }
    }
}

/// Dashboard summary data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_endpoints: i64,
    pub online_endpoints: i64,
    pub offline_endpoints: i64,
    pub warning_endpoints: i64,
    pub critical_endpoints: i64,
    pub total_checks: i64,
    pub enabled_checks: i64,
    pub recent_results: Vec<RecentCheckResult>,
}

/// Recent check result for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentCheckResult {
    pub endpoint_hostname: String,
    pub check_name: String,
    pub status: CheckStatus,
    pub message: Option<String>,
    pub collected_at: DateTime<Utc>,
}
