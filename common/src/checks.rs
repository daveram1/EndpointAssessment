use serde::{Deserialize, Serialize};

/// Check type identifier (for database storage)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckTypeId {
    FileExists,
    FileContent,
    RegistryKey,
    ConfigSetting,
    ProcessRunning,
    PortOpen,
    CommandOutput,
}

impl std::fmt::Display for CheckTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CheckTypeId::FileExists => "file_exists",
            CheckTypeId::FileContent => "file_content",
            CheckTypeId::RegistryKey => "registry_key",
            CheckTypeId::ConfigSetting => "config_setting",
            CheckTypeId::ProcessRunning => "process_running",
            CheckTypeId::PortOpen => "port_open",
            CheckTypeId::CommandOutput => "command_output",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for CheckTypeId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file_exists" => Ok(CheckTypeId::FileExists),
            "file_content" => Ok(CheckTypeId::FileContent),
            "registry_key" => Ok(CheckTypeId::RegistryKey),
            "config_setting" => Ok(CheckTypeId::ConfigSetting),
            "process_running" => Ok(CheckTypeId::ProcessRunning),
            "port_open" => Ok(CheckTypeId::PortOpen),
            "command_output" => Ok(CheckTypeId::CommandOutput),
            _ => Err(format!("Unknown check type: {}", s)),
        }
    }
}

/// Parameters for file_exists check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExistsParams {
    pub path: String,
}

/// Parameters for file_content check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContentParams {
    pub path: String,
    pub pattern: String,
    #[serde(default = "default_true")]
    pub should_match: bool,
}

fn default_true() -> bool {
    true
}

/// Parameters for registry_key check (Windows only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryKeyParams {
    pub path: String,
    pub value_name: Option<String>,
    pub expected: Option<String>,
}

/// Parameters for config_setting check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSettingParams {
    pub file: String,
    pub key: String,
    pub expected: String,
}

/// Parameters for process_running check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessRunningParams {
    pub name: String,
}

/// Parameters for port_open check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortOpenParams {
    pub port: u16,
}

/// Parameters for command_output check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutputParams {
    pub command: String,
    pub expected_pattern: String,
}

/// Helper to get check type description
pub fn check_type_description(type_id: CheckTypeId) -> &'static str {
    match type_id {
        CheckTypeId::FileExists => "Check if a file exists at the specified path",
        CheckTypeId::FileContent => "Check if file content matches a pattern",
        CheckTypeId::RegistryKey => "Check Windows registry key value (Windows only)",
        CheckTypeId::ConfigSetting => "Check configuration file setting value",
        CheckTypeId::ProcessRunning => "Check if a process is running",
        CheckTypeId::PortOpen => "Check if a port is open/listening",
        CheckTypeId::CommandOutput => "Check command output matches a pattern",
    }
}
