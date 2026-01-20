use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Status of an endpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EndpointStatus {
    Online,
    #[default]
    Offline,
    Warning,
    Critical,
}

impl std::fmt::Display for EndpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndpointStatus::Online => write!(f, "online"),
            EndpointStatus::Offline => write!(f, "offline"),
            EndpointStatus::Warning => write!(f, "warning"),
            EndpointStatus::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for EndpointStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "online" => Ok(EndpointStatus::Online),
            "offline" => Ok(EndpointStatus::Offline),
            "warning" => Ok(EndpointStatus::Warning),
            "critical" => Ok(EndpointStatus::Critical),
            _ => Err(format!("Unknown endpoint status: {}", s)),
        }
    }
}

/// Endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: Uuid,
    pub hostname: String,
    pub os: Option<String>,
    pub os_version: Option<String>,
    pub agent_version: Option<String>,
    pub ip_addresses: Vec<String>,
    pub last_seen: Option<DateTime<Utc>>,
    pub status: EndpointStatus,
    pub created_at: DateTime<Utc>,
}

/// Severity level for checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Low => write!(f, "low"),
            Severity::Medium => write!(f, "medium"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(Severity::Info),
            "low" => Ok(Severity::Low),
            "medium" => Ok(Severity::Medium),
            "high" => Ok(Severity::High),
            "critical" => Ok(Severity::Critical),
            _ => Err(format!("Unknown severity: {}", s)),
        }
    }
}

/// Check type with parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CheckType {
    FileExists {
        path: String,
    },
    FileContent {
        path: String,
        pattern: String,
        should_match: bool,
    },
    RegistryKey {
        path: String,
        value_name: Option<String>,
        expected: Option<String>,
    },
    ConfigSetting {
        file: String,
        key: String,
        expected: String,
    },
    ProcessRunning {
        name: String,
    },
    PortOpen {
        port: u16,
    },
    CommandOutput {
        command: String,
        expected_pattern: String,
    },
}

impl CheckType {
    pub fn type_name(&self) -> &'static str {
        match self {
            CheckType::FileExists { .. } => "file_exists",
            CheckType::FileContent { .. } => "file_content",
            CheckType::RegistryKey { .. } => "registry_key",
            CheckType::ConfigSetting { .. } => "config_setting",
            CheckType::ProcessRunning { .. } => "process_running",
            CheckType::PortOpen { .. } => "port_open",
            CheckType::CommandOutput { .. } => "command_output",
        }
    }
}

/// Check definition stored in database
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CheckDefinition {
    pub id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub check_type: CheckType,
    pub severity: Severity,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Status of a check result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Fail,
    Error,
    Skipped,
}

impl std::fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckStatus::Pass => write!(f, "pass"),
            CheckStatus::Fail => write!(f, "fail"),
            CheckStatus::Error => write!(f, "error"),
            CheckStatus::Skipped => write!(f, "skipped"),
        }
    }
}

impl std::str::FromStr for CheckStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pass" => Ok(CheckStatus::Pass),
            "fail" => Ok(CheckStatus::Fail),
            "error" => Ok(CheckStatus::Error),
            "skipped" => Ok(CheckStatus::Skipped),
            _ => Err(format!("Unknown check status: {}", s)),
        }
    }
}

/// Result of a check execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub check_id: Uuid,
    pub status: CheckStatus,
    pub message: Option<String>,
    pub collected_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Information about a running process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
}

/// Information about installed software
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInfo {
    pub name: String,
    pub version: Option<String>,
    pub publisher: Option<String>,
}

/// System snapshot collected by agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub endpoint_id: Uuid,
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

/// Admin user for web UI access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: AdminRole,
    pub created_at: DateTime<Utc>,
}

/// Admin user role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AdminRole {
    Admin,
    #[default]
    Viewer,
}

impl std::fmt::Display for AdminRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdminRole::Admin => write!(f, "admin"),
            AdminRole::Viewer => write!(f, "viewer"),
        }
    }
}

impl std::str::FromStr for AdminRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(AdminRole::Admin),
            "viewer" => Ok(AdminRole::Viewer),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}
