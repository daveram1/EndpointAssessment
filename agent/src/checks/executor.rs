use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;

use common::AgentCheckDefinition;
use regex::Regex;
use sysinfo::System;

use super::types::*;

pub struct CheckExecutor {
    system: System,
}

impl CheckExecutor {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    pub fn execute(&mut self, check: &AgentCheckDefinition) -> CheckExecutionResult {
        self.system.refresh_all();

        match check.check_type.as_str() {
            "file_exists" => self.execute_file_exists(&check.parameters),
            "file_content" => self.execute_file_content(&check.parameters),
            "registry_key" => self.execute_registry_key(&check.parameters),
            "config_setting" => self.execute_config_setting(&check.parameters),
            "process_running" => self.execute_process_running(&check.parameters),
            "port_open" => self.execute_port_open(&check.parameters),
            "command_output" => self.execute_command_output(&check.parameters),
            _ => CheckExecutionResult::error(format!("Unknown check type: {}", check.check_type)),
        }
    }

    fn execute_file_exists(&self, params: &serde_json::Value) -> CheckExecutionResult {
        let params: FileExistsParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return CheckExecutionResult::error(format!("Invalid parameters: {}", e)),
        };

        let path = Path::new(&params.path);
        if path.exists() {
            CheckExecutionResult::pass(Some(format!("File exists: {}", params.path)))
        } else {
            CheckExecutionResult::fail(format!("File not found: {}", params.path))
        }
    }

    fn execute_file_content(&self, params: &serde_json::Value) -> CheckExecutionResult {
        let params: FileContentParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return CheckExecutionResult::error(format!("Invalid parameters: {}", e)),
        };

        let content = match fs::read_to_string(&params.path) {
            Ok(c) => c,
            Err(e) => {
                return CheckExecutionResult::error(format!(
                    "Failed to read file {}: {}",
                    params.path, e
                ))
            }
        };

        let regex = match Regex::new(&params.pattern) {
            Ok(r) => r,
            Err(e) => {
                return CheckExecutionResult::error(format!("Invalid regex pattern: {}", e))
            }
        };

        let matches = regex.is_match(&content);

        if matches == params.should_match {
            CheckExecutionResult::pass(Some(format!(
                "Pattern {} in file",
                if matches { "found" } else { "not found" }
            )))
        } else {
            CheckExecutionResult::fail(format!(
                "Pattern {} in file (expected {})",
                if matches { "found" } else { "not found" },
                if params.should_match {
                    "match"
                } else {
                    "no match"
                }
            ))
        }
    }

    #[cfg(target_os = "windows")]
    fn execute_registry_key(&self, params: &serde_json::Value) -> CheckExecutionResult {
        use winreg::enums::*;
        use winreg::RegKey;

        let params: RegistryKeyParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return CheckExecutionResult::error(format!("Invalid parameters: {}", e)),
        };

        // Parse the registry path
        let (hkey, subkey) = if params.path.starts_with("HKEY_LOCAL_MACHINE\\")
            || params.path.starts_with("HKLM\\")
        {
            let path = params
                .path
                .trim_start_matches("HKEY_LOCAL_MACHINE\\")
                .trim_start_matches("HKLM\\");
            (HKEY_LOCAL_MACHINE, path)
        } else if params.path.starts_with("HKEY_CURRENT_USER\\")
            || params.path.starts_with("HKCU\\")
        {
            let path = params
                .path
                .trim_start_matches("HKEY_CURRENT_USER\\")
                .trim_start_matches("HKCU\\");
            (HKEY_CURRENT_USER, path)
        } else {
            return CheckExecutionResult::error(format!("Unsupported registry hive in path: {}", params.path));
        };

        let hkey = RegKey::predef(hkey);
        let key = match hkey.open_subkey(subkey) {
            Ok(k) => k,
            Err(e) => {
                return CheckExecutionResult::fail(format!(
                    "Registry key not found: {} ({})",
                    params.path, e
                ))
            }
        };

        if let Some(value_name) = &params.value_name {
            let value: Result<String, _> = key.get_value(value_name);
            match value {
                Ok(v) => {
                    if let Some(expected) = &params.expected {
                        if &v == expected {
                            CheckExecutionResult::pass(Some(format!(
                                "Registry value matches: {} = {}",
                                value_name, v
                            )))
                        } else {
                            CheckExecutionResult::fail(format!(
                                "Registry value mismatch: {} = {} (expected {})",
                                value_name, v, expected
                            ))
                        }
                    } else {
                        CheckExecutionResult::pass(Some(format!(
                            "Registry value exists: {} = {}",
                            value_name, v
                        )))
                    }
                }
                Err(e) => CheckExecutionResult::fail(format!(
                    "Registry value not found: {} ({})",
                    value_name, e
                )),
            }
        } else {
            CheckExecutionResult::pass(Some(format!("Registry key exists: {}", params.path)))
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn execute_registry_key(&self, _params: &serde_json::Value) -> CheckExecutionResult {
        CheckExecutionResult::skipped("Registry checks are only available on Windows")
    }

    fn execute_config_setting(&self, params: &serde_json::Value) -> CheckExecutionResult {
        let params: ConfigSettingParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return CheckExecutionResult::error(format!("Invalid parameters: {}", e)),
        };

        let content = match fs::read_to_string(&params.file) {
            Ok(c) => c,
            Err(e) => {
                return CheckExecutionResult::error(format!(
                    "Failed to read config file {}: {}",
                    params.file, e
                ))
            }
        };

        // Try different config file formats

        // INI-style: key=value or key = value
        let pattern = format!(r"(?m)^\s*{}\s*[=:]\s*(.*)$", regex::escape(&params.key));
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(caps) = re.captures(&content) {
                let value = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                if value == params.expected {
                    return CheckExecutionResult::pass(Some(format!(
                        "Config setting matches: {} = {}",
                        params.key, value
                    )));
                } else {
                    return CheckExecutionResult::fail(format!(
                        "Config setting mismatch: {} = {} (expected {})",
                        params.key, value, params.expected
                    ));
                }
            }
        }

        CheckExecutionResult::fail(format!(
            "Config setting not found: {} in {}",
            params.key, params.file
        ))
    }

    fn execute_process_running(&self, params: &serde_json::Value) -> CheckExecutionResult {
        let params: ProcessRunningParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return CheckExecutionResult::error(format!("Invalid parameters: {}", e)),
        };

        let name_lower = params.name.to_lowercase();

        for (_pid, process) in self.system.processes() {
            let process_name = process.name().to_string_lossy().to_lowercase();
            if process_name.contains(&name_lower) {
                return CheckExecutionResult::pass(Some(format!(
                    "Process is running: {}",
                    process.name().to_string_lossy()
                )));
            }
        }

        CheckExecutionResult::fail(format!("Process not running: {}", params.name))
    }

    fn execute_port_open(&self, params: &serde_json::Value) -> CheckExecutionResult {
        let params: PortOpenParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return CheckExecutionResult::error(format!("Invalid parameters: {}", e)),
        };

        // Try to bind to the port - if it fails, something is listening
        match TcpListener::bind(("127.0.0.1", params.port)) {
            Ok(_) => {
                // We could bind, so nothing is listening
                CheckExecutionResult::fail(format!("Port {} is not open/listening", params.port))
            }
            Err(_) => {
                // Couldn't bind, so something is using the port
                CheckExecutionResult::pass(Some(format!("Port {} is open/listening", params.port)))
            }
        }
    }

    fn execute_command_output(&self, params: &serde_json::Value) -> CheckExecutionResult {
        let params: CommandOutputParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => return CheckExecutionResult::error(format!("Invalid parameters: {}", e)),
        };

        #[cfg(target_os = "windows")]
        let output = Command::new("cmd").args(["/C", &params.command]).output();

        #[cfg(not(target_os = "windows"))]
        let output = Command::new("sh").args(["-c", &params.command]).output();

        let output = match output {
            Ok(o) => o,
            Err(e) => {
                return CheckExecutionResult::error(format!("Failed to execute command: {}", e))
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        let regex = match Regex::new(&params.expected_pattern) {
            Ok(r) => r,
            Err(e) => {
                return CheckExecutionResult::error(format!("Invalid regex pattern: {}", e))
            }
        };

        if regex.is_match(&combined) {
            CheckExecutionResult::pass(Some("Command output matches expected pattern".to_string()))
        } else {
            CheckExecutionResult::fail(format!(
                "Command output does not match pattern. Output: {}",
                combined.chars().take(200).collect::<String>()
            ))
        }
    }
}

impl Default for CheckExecutor {
    fn default() -> Self {
        Self::new()
    }
}
