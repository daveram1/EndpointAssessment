use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub agent_secret: String,
    #[serde(default = "default_interval")]
    pub collection_interval_secs: u64,
    #[serde(default)]
    pub hostname_override: Option<String>,
}

fn default_interval() -> u64 {
    300 // 5 minutes
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?;

        config.try_deserialize()
    }

    pub fn from_args(server_url: String, agent_secret: String) -> Self {
        Self {
            server_url,
            agent_secret,
            collection_interval_secs: default_interval(),
            hostname_override: None,
        }
    }
}
