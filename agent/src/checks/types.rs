use common::{AgentCheckDefinition, CheckStatus};
use serde::Deserialize;

#[derive(Debug)]
pub struct CheckExecutionResult {
    pub status: CheckStatus,
    pub message: Option<String>,
}

impl CheckExecutionResult {
    pub fn pass(message: Option<String>) -> Self {
        Self {
            status: CheckStatus::Pass,
            message,
        }
    }

    pub fn fail(message: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Fail,
            message: Some(message.into()),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Error,
            message: Some(message.into()),
        }
    }

    pub fn skipped(message: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Skipped,
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FileExistsParams {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct FileContentParams {
    pub path: String,
    pub pattern: String,
    #[serde(default = "default_true")]
    pub should_match: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct RegistryKeyParams {
    pub path: String,
    pub value_name: Option<String>,
    pub expected: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigSettingParams {
    pub file: String,
    pub key: String,
    pub expected: String,
}

#[derive(Debug, Deserialize)]
pub struct ProcessRunningParams {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct PortOpenParams {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct CommandOutputParams {
    pub command: String,
    pub expected_pattern: String,
}
