use chrono::{DateTime, Utc};
use common::{ProcessInfo, SoftwareInfo, SystemSnapshot};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_snapshot(
    pool: &PgPool,
    endpoint_id: Uuid,
    cpu_usage: f32,
    memory_total: i64,
    memory_used: i64,
    disk_total: i64,
    disk_used: i64,
    processes: &[ProcessInfo],
    open_ports: &[u16],
    installed_software: &[SoftwareInfo],
    collected_at: DateTime<Utc>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    let processes_json = serde_json::to_value(processes).unwrap_or_default();
    let ports_json = serde_json::to_value(open_ports).unwrap_or_default();
    let software_json = serde_json::to_value(installed_software).unwrap_or_default();

    sqlx::query!(
        r#"
        INSERT INTO system_snapshots (id, endpoint_id, cpu_usage, memory_total, memory_used, disk_total, disk_used, processes, open_ports, installed_software, collected_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
        id,
        endpoint_id,
        cpu_usage,
        memory_total,
        memory_used,
        disk_total,
        disk_used,
        processes_json,
        ports_json,
        software_json,
        collected_at,
    )
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn get_latest_snapshot(pool: &PgPool, endpoint_id: Uuid) -> Result<Option<SystemSnapshot>, sqlx::Error> {
    let row = sqlx::query_as!(
        SnapshotRow,
        r#"
        SELECT id, endpoint_id, cpu_usage, memory_total, memory_used, disk_total, disk_used, processes, open_ports, installed_software, collected_at
        FROM system_snapshots
        WHERE endpoint_id = $1
        ORDER BY collected_at DESC
        LIMIT 1
        "#,
        endpoint_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.into_snapshot()))
}

pub async fn get_snapshots_for_endpoint(
    pool: &PgPool,
    endpoint_id: Uuid,
    limit: i64,
) -> Result<Vec<SystemSnapshot>, sqlx::Error> {
    let rows = sqlx::query_as!(
        SnapshotRow,
        r#"
        SELECT id, endpoint_id, cpu_usage, memory_total, memory_used, disk_total, disk_used, processes, open_ports, installed_software, collected_at
        FROM system_snapshots
        WHERE endpoint_id = $1
        ORDER BY collected_at DESC
        LIMIT $2
        "#,
        endpoint_id,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into_snapshot()).collect())
}

pub async fn cleanup_old_snapshots(pool: &PgPool, days_to_keep: i64) -> Result<u64, sqlx::Error> {
    let threshold = Utc::now() - chrono::Duration::days(days_to_keep);

    let result = sqlx::query!(
        r#"
        DELETE FROM system_snapshots WHERE collected_at < $1
        "#,
        threshold
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

struct SnapshotRow {
    id: Uuid,
    endpoint_id: Uuid,
    cpu_usage: Option<f32>,
    memory_total: Option<i64>,
    memory_used: Option<i64>,
    disk_total: Option<i64>,
    disk_used: Option<i64>,
    processes: Option<serde_json::Value>,
    open_ports: Option<serde_json::Value>,
    installed_software: Option<serde_json::Value>,
    collected_at: DateTime<Utc>,
}

impl SnapshotRow {
    fn into_snapshot(self) -> SystemSnapshot {
        let processes: Vec<ProcessInfo> = self
            .processes
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let open_ports: Vec<u16> = self
            .open_ports
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let installed_software: Vec<SoftwareInfo> = self
            .installed_software
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        SystemSnapshot {
            endpoint_id: self.endpoint_id,
            collected_at: self.collected_at,
            cpu_usage: self.cpu_usage.unwrap_or(0.0),
            memory_total: self.memory_total.unwrap_or(0) as u64,
            memory_used: self.memory_used.unwrap_or(0) as u64,
            disk_total: self.disk_total.unwrap_or(0) as u64,
            disk_used: self.disk_used.unwrap_or(0) as u64,
            processes,
            open_ports,
            installed_software,
        }
    }
}
