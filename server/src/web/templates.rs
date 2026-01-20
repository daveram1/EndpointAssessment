use askama::Template;
use common::{CheckStatus, Endpoint, EndpointStatus, Severity, SystemSnapshot};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate {
    pub title: String,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub title: String,
    pub total_endpoints: i64,
    pub online_endpoints: i64,
    pub offline_endpoints: i64,
    pub warning_endpoints: i64,
    pub critical_endpoints: i64,
    pub total_checks: i64,
    pub enabled_checks: i64,
    pub recent_results: Vec<RecentResultView>,
}

pub struct RecentResultView {
    pub endpoint_hostname: String,
    pub check_name: String,
    pub status: CheckStatus,
    pub message: Option<String>,
    pub collected_at: String,
}

impl RecentResultView {
    pub fn status_class(&self) -> &'static str {
        match self.status {
            CheckStatus::Pass => "success",
            CheckStatus::Fail => "danger",
            CheckStatus::Error => "warning",
            CheckStatus::Skipped => "secondary",
        }
    }
}

#[derive(Template)]
#[template(path = "endpoints.html")]
pub struct EndpointsTemplate {
    pub title: String,
    pub endpoints: Vec<EndpointView>,
}

pub struct EndpointView {
    pub id: Uuid,
    pub hostname: String,
    pub os: String,
    pub agent_version: String,
    pub ip_addresses: String,
    pub last_seen: String,
    pub status: EndpointStatus,
}

impl EndpointView {
    pub fn status_class(&self) -> &'static str {
        match self.status {
            EndpointStatus::Online => "success",
            EndpointStatus::Offline => "secondary",
            EndpointStatus::Warning => "warning",
            EndpointStatus::Critical => "danger",
        }
    }
}

impl From<Endpoint> for EndpointView {
    fn from(e: Endpoint) -> Self {
        Self {
            id: e.id,
            hostname: e.hostname,
            os: format!(
                "{} {}",
                e.os.unwrap_or_default(),
                e.os_version.unwrap_or_default()
            ),
            agent_version: e.agent_version.unwrap_or_else(|| "Unknown".to_string()),
            ip_addresses: e.ip_addresses.join(", "),
            last_seen: e
                .last_seen
                .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| "Never".to_string()),
            status: e.status,
        }
    }
}

#[derive(Template)]
#[template(path = "endpoint_detail.html")]
pub struct EndpointDetailTemplate {
    pub title: String,
    pub endpoint: EndpointView,
    pub snapshot: Option<SnapshotView>,
    pub check_results: Vec<CheckResultView>,
}

pub struct SnapshotView {
    pub cpu_usage: String,
    pub memory_used: String,
    pub memory_total: String,
    pub memory_percent: String,
    pub disk_used: String,
    pub disk_total: String,
    pub disk_percent: String,
    pub process_count: usize,
    pub open_ports: String,
    pub collected_at: String,
}

impl From<SystemSnapshot> for SnapshotView {
    fn from(s: SystemSnapshot) -> Self {
        let memory_percent = if s.memory_total > 0 {
            (s.memory_used as f64 / s.memory_total as f64 * 100.0) as u64
        } else {
            0
        };

        let disk_percent = if s.disk_total > 0 {
            (s.disk_used as f64 / s.disk_total as f64 * 100.0) as u64
        } else {
            0
        };

        Self {
            cpu_usage: format!("{:.1}%", s.cpu_usage),
            memory_used: format_bytes(s.memory_used),
            memory_total: format_bytes(s.memory_total),
            memory_percent: format!("{}%", memory_percent),
            disk_used: format_bytes(s.disk_used),
            disk_total: format_bytes(s.disk_total),
            disk_percent: format!("{}%", disk_percent),
            process_count: s.processes.len(),
            open_ports: s
                .open_ports
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            collected_at: s.collected_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub struct CheckResultView {
    pub check_id: Uuid,
    pub check_name: String,
    pub status: CheckStatus,
    pub message: Option<String>,
    pub collected_at: String,
}

impl CheckResultView {
    pub fn status_class(&self) -> &'static str {
        match self.status {
            CheckStatus::Pass => "success",
            CheckStatus::Fail => "danger",
            CheckStatus::Error => "warning",
            CheckStatus::Skipped => "secondary",
        }
    }
}

#[derive(Template)]
#[template(path = "checks.html")]
pub struct ChecksTemplate {
    pub title: String,
    pub checks: Vec<CheckDefView>,
}

pub struct CheckDefView {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub check_type: String,
    pub severity: Severity,
    pub enabled: bool,
    pub updated_at: String,
}

impl CheckDefView {
    pub fn severity_class(&self) -> &'static str {
        match self.severity {
            Severity::Info => "info",
            Severity::Low => "secondary",
            Severity::Medium => "warning",
            Severity::High => "danger",
            Severity::Critical => "dark",
        }
    }
}

#[derive(Template)]
#[template(path = "check_form.html")]
pub struct CheckFormTemplate {
    pub title: String,
    pub check: Option<CheckDefView>,
    pub parameters_json: String,
}

#[derive(Template)]
#[template(path = "reports.html")]
pub struct ReportsTemplate {
    pub title: String,
    pub total_results: i64,
    pub passed: i64,
    pub failed: i64,
    pub errors: i64,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub title: String,
    pub error: Option<String>,
}
