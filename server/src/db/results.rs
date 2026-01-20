use chrono::{DateTime, Utc};
use common::CheckStatus;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CheckResultRow {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub check_id: Uuid,
    pub status: String,
    pub message: Option<String>,
    pub collected_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub async fn create_result(
    pool: &PgPool,
    endpoint_id: Uuid,
    check_id: Uuid,
    status: CheckStatus,
    message: Option<&str>,
    collected_at: DateTime<Utc>,
) -> Result<CheckResultRow, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let status_str = status.to_string();

    sqlx::query_as!(
        CheckResultRow,
        r#"
        INSERT INTO check_results (id, endpoint_id, check_id, status, message, collected_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, endpoint_id, check_id, status, message, collected_at, created_at
        "#,
        id,
        endpoint_id,
        check_id,
        status_str,
        message,
        collected_at,
        now,
    )
    .fetch_one(pool)
    .await
}

pub async fn get_results_for_endpoint(
    pool: &PgPool,
    endpoint_id: Uuid,
    limit: i64,
) -> Result<Vec<CheckResultRow>, sqlx::Error> {
    sqlx::query_as!(
        CheckResultRow,
        r#"
        SELECT id, endpoint_id, check_id, status, message, collected_at, created_at
        FROM check_results
        WHERE endpoint_id = $1
        ORDER BY collected_at DESC
        LIMIT $2
        "#,
        endpoint_id,
        limit
    )
    .fetch_all(pool)
    .await
}

pub async fn get_results_for_check(
    pool: &PgPool,
    check_id: Uuid,
    limit: i64,
) -> Result<Vec<CheckResultRow>, sqlx::Error> {
    sqlx::query_as!(
        CheckResultRow,
        r#"
        SELECT id, endpoint_id, check_id, status, message, collected_at, created_at
        FROM check_results
        WHERE check_id = $1
        ORDER BY collected_at DESC
        LIMIT $2
        "#,
        check_id,
        limit
    )
    .fetch_all(pool)
    .await
}

pub async fn get_latest_results_for_endpoint(
    pool: &PgPool,
    endpoint_id: Uuid,
) -> Result<Vec<LatestResultRow>, sqlx::Error> {
    sqlx::query_as!(
        LatestResultRow,
        r#"
        SELECT DISTINCT ON (cr.check_id)
            cr.id,
            cr.endpoint_id,
            cr.check_id,
            cd.name as check_name,
            cr.status,
            cr.message,
            cr.collected_at
        FROM check_results cr
        JOIN check_definitions cd ON cd.id = cr.check_id
        WHERE cr.endpoint_id = $1
        ORDER BY cr.check_id, cr.collected_at DESC
        "#,
        endpoint_id
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone)]
pub struct LatestResultRow {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub check_id: Uuid,
    pub check_name: String,
    pub status: String,
    pub message: Option<String>,
    pub collected_at: DateTime<Utc>,
}

pub async fn get_recent_results(pool: &PgPool, limit: i64) -> Result<Vec<RecentResultRow>, sqlx::Error> {
    sqlx::query_as!(
        RecentResultRow,
        r#"
        SELECT
            cr.id,
            e.hostname as endpoint_hostname,
            cd.name as check_name,
            cr.status,
            cr.message,
            cr.collected_at
        FROM check_results cr
        JOIN endpoints e ON e.id = cr.endpoint_id
        JOIN check_definitions cd ON cd.id = cr.check_id
        ORDER BY cr.collected_at DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone)]
pub struct RecentResultRow {
    pub id: Uuid,
    pub endpoint_hostname: String,
    pub check_name: String,
    pub status: String,
    pub message: Option<String>,
    pub collected_at: DateTime<Utc>,
}

pub async fn get_result_stats(pool: &PgPool) -> Result<ResultStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'pass') as passed,
            COUNT(*) FILTER (WHERE status = 'fail') as failed,
            COUNT(*) FILTER (WHERE status = 'error') as errors
        FROM check_results
        WHERE collected_at > NOW() - INTERVAL '24 hours'
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(ResultStats {
        total: row.total.unwrap_or(0),
        passed: row.passed.unwrap_or(0),
        failed: row.failed.unwrap_or(0),
        errors: row.errors.unwrap_or(0),
    })
}

#[derive(Debug)]
pub struct ResultStats {
    pub total: i64,
    pub passed: i64,
    pub failed: i64,
    pub errors: i64,
}
