use chrono::{DateTime, Utc};
use common::{Endpoint, EndpointStatus};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_endpoint(
    pool: &PgPool,
    hostname: &str,
    os: &str,
    os_version: &str,
    agent_version: &str,
    ip_addresses: &[String],
) -> Result<Endpoint, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let ip_json = serde_json::to_value(ip_addresses).unwrap_or_default();

    sqlx::query_as!(
        EndpointRow,
        r#"
        INSERT INTO endpoints (id, hostname, os, os_version, agent_version, ip_addresses, last_seen, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'online', $7)
        ON CONFLICT (hostname) DO UPDATE SET
            os = EXCLUDED.os,
            os_version = EXCLUDED.os_version,
            agent_version = EXCLUDED.agent_version,
            ip_addresses = EXCLUDED.ip_addresses,
            last_seen = EXCLUDED.last_seen,
            status = 'online'
        RETURNING id, hostname, os, os_version, agent_version, ip_addresses, last_seen, status, created_at
        "#,
        id,
        hostname,
        os,
        os_version,
        agent_version,
        ip_json,
        now,
    )
    .fetch_one(pool)
    .await
    .map(|row| row.into_endpoint())
}

pub async fn get_endpoint_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Endpoint>, sqlx::Error> {
    sqlx::query_as!(
        EndpointRow,
        r#"
        SELECT id, hostname, os, os_version, agent_version, ip_addresses, last_seen, status, created_at
        FROM endpoints WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
    .map(|opt| opt.map(|row| row.into_endpoint()))
}

pub async fn get_endpoint_by_hostname(pool: &PgPool, hostname: &str) -> Result<Option<Endpoint>, sqlx::Error> {
    sqlx::query_as!(
        EndpointRow,
        r#"
        SELECT id, hostname, os, os_version, agent_version, ip_addresses, last_seen, status, created_at
        FROM endpoints WHERE hostname = $1
        "#,
        hostname
    )
    .fetch_optional(pool)
    .await
    .map(|opt| opt.map(|row| row.into_endpoint()))
}

pub async fn list_endpoints(pool: &PgPool) -> Result<Vec<Endpoint>, sqlx::Error> {
    sqlx::query_as!(
        EndpointRow,
        r#"
        SELECT id, hostname, os, os_version, agent_version, ip_addresses, last_seen, status, created_at
        FROM endpoints ORDER BY hostname
        "#
    )
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|row| row.into_endpoint()).collect())
}

pub async fn update_endpoint_heartbeat(
    pool: &PgPool,
    id: Uuid,
    status: EndpointStatus,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let status_str = status.to_string();

    sqlx::query!(
        r#"
        UPDATE endpoints SET last_seen = $1, status = $2 WHERE id = $3
        "#,
        now,
        status_str,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_offline_endpoints(pool: &PgPool, threshold_minutes: i64) -> Result<u64, sqlx::Error> {
    let threshold = Utc::now() - chrono::Duration::minutes(threshold_minutes);

    let result = sqlx::query!(
        r#"
        UPDATE endpoints SET status = 'offline'
        WHERE last_seen < $1 AND status != 'offline'
        "#,
        threshold
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn delete_endpoint(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM endpoints WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_endpoint_counts(pool: &PgPool) -> Result<EndpointCounts, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'online') as online,
            COUNT(*) FILTER (WHERE status = 'offline') as offline,
            COUNT(*) FILTER (WHERE status = 'warning') as warning,
            COUNT(*) FILTER (WHERE status = 'critical') as critical
        FROM endpoints
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(EndpointCounts {
        total: row.total.unwrap_or(0),
        online: row.online.unwrap_or(0),
        offline: row.offline.unwrap_or(0),
        warning: row.warning.unwrap_or(0),
        critical: row.critical.unwrap_or(0),
    })
}

#[derive(Debug)]
pub struct EndpointCounts {
    pub total: i64,
    pub online: i64,
    pub offline: i64,
    pub warning: i64,
    pub critical: i64,
}

struct EndpointRow {
    id: Uuid,
    hostname: String,
    os: Option<String>,
    os_version: Option<String>,
    agent_version: Option<String>,
    ip_addresses: Option<serde_json::Value>,
    last_seen: Option<DateTime<Utc>>,
    status: Option<String>,
    created_at: Option<DateTime<Utc>>,
}

impl EndpointRow {
    fn into_endpoint(self) -> Endpoint {
        let ip_addresses: Vec<String> = self
            .ip_addresses
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let status = self
            .status
            .and_then(|s| s.parse().ok())
            .unwrap_or(EndpointStatus::Offline);

        Endpoint {
            id: self.id,
            hostname: self.hostname,
            os: self.os,
            os_version: self.os_version,
            agent_version: self.agent_version,
            ip_addresses,
            last_seen: self.last_seen,
            status,
            created_at: self.created_at.unwrap_or_else(Utc::now),
        }
    }
}
