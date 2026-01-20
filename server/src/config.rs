use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub database_url: String,
    #[serde(default = "default_agent_secret")]
    pub agent_secret: String,
    #[serde(default = "default_session_secret")]
    pub session_secret: String,
    #[serde(default = "default_offline_threshold")]
    pub offline_threshold_minutes: i64,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_agent_secret() -> String {
    "change-me-in-production".to_string()
}

fn default_session_secret() -> String {
    "session-secret-change-me".to_string()
}

fn default_offline_threshold() -> i64 {
    10
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?;

        config.try_deserialize()
    }

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid socket address")
    }
}
