use anyhow::{Context, Result};
use common::{
    AgentCheckResult, ChecksResponse, HeartbeatRequest, HeartbeatResponse, RegisterRequest,
    RegisterResponse, SubmitResultsRequest, SubmitResultsResponse, SystemSnapshotData,
};
use reqwest::Client;
use uuid::Uuid;

pub struct ServerClient {
    client: Client,
    base_url: String,
    agent_secret: String,
}

impl ServerClient {
    pub fn new(base_url: &str, agent_secret: &str) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            agent_secret: agent_secret.to_string(),
        }
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<RegisterResponse> {
        let url = format!("{}/api/agent/register", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("X-Agent-Secret", &self.agent_secret)
            .json(&request)
            .send()
            .await
            .context("Failed to send registration request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Registration failed: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse registration response")
    }

    pub async fn heartbeat(
        &self,
        endpoint_id: Uuid,
        snapshot: SystemSnapshotData,
    ) -> Result<HeartbeatResponse> {
        let url = format!("{}/api/agent/heartbeat", self.base_url);

        let request = HeartbeatRequest {
            endpoint_id,
            snapshot,
        };

        let response = self
            .client
            .post(&url)
            .header("X-Agent-Secret", &self.agent_secret)
            .json(&request)
            .send()
            .await
            .context("Failed to send heartbeat")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Heartbeat failed: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse heartbeat response")
    }

    pub async fn get_checks(&self) -> Result<ChecksResponse> {
        let url = format!("{}/api/agent/checks", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("X-Agent-Secret", &self.agent_secret)
            .send()
            .await
            .context("Failed to fetch checks")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get checks: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse checks response")
    }

    pub async fn submit_results(
        &self,
        endpoint_id: Uuid,
        results: Vec<AgentCheckResult>,
    ) -> Result<SubmitResultsResponse> {
        let url = format!("{}/api/agent/results", self.base_url);

        let request = SubmitResultsRequest {
            endpoint_id,
            results,
        };

        let response = self
            .client
            .post(&url)
            .header("X-Agent-Secret", &self.agent_secret)
            .json(&request)
            .send()
            .await
            .context("Failed to submit results")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to submit results: {} - {}", status, text);
        }

        response
            .json()
            .await
            .context("Failed to parse submit results response")
    }
}
