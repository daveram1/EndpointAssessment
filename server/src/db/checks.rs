use chrono::{DateTime, Utc};
use common::Severity;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CheckDefinitionRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub check_type: String,
    pub parameters: serde_json::Value,
    pub severity: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_check(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
    check_type: &str,
    parameters: serde_json::Value,
    severity: Severity,
    enabled: bool,
) -> Result<CheckDefinitionRow, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let severity_str = severity.to_string();

    sqlx::query_as!(
        CheckDefinitionRow,
        r#"
        INSERT INTO check_definitions (id, name, description, check_type, parameters, severity, enabled, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
        RETURNING id, name, description, check_type, parameters, severity, enabled, created_at, updated_at
        "#,
        id,
        name,
        description,
        check_type,
        parameters,
        severity_str,
        enabled,
        now,
    )
    .fetch_one(pool)
    .await
}

pub async fn get_check_by_id(pool: &PgPool, id: Uuid) -> Result<Option<CheckDefinitionRow>, sqlx::Error> {
    sqlx::query_as!(
        CheckDefinitionRow,
        r#"
        SELECT id, name, description, check_type, parameters, severity, enabled, created_at, updated_at
        FROM check_definitions WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_checks(pool: &PgPool) -> Result<Vec<CheckDefinitionRow>, sqlx::Error> {
    sqlx::query_as!(
        CheckDefinitionRow,
        r#"
        SELECT id, name, description, check_type, parameters, severity, enabled, created_at, updated_at
        FROM check_definitions ORDER BY name
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn list_enabled_checks(pool: &PgPool) -> Result<Vec<CheckDefinitionRow>, sqlx::Error> {
    sqlx::query_as!(
        CheckDefinitionRow,
        r#"
        SELECT id, name, description, check_type, parameters, severity, enabled, created_at, updated_at
        FROM check_definitions WHERE enabled = true ORDER BY name
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn update_check(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    description: Option<&str>,
    check_type: &str,
    parameters: serde_json::Value,
    severity: Severity,
    enabled: bool,
) -> Result<Option<CheckDefinitionRow>, sqlx::Error> {
    let now = Utc::now();
    let severity_str = severity.to_string();

    sqlx::query_as!(
        CheckDefinitionRow,
        r#"
        UPDATE check_definitions SET
            name = $2,
            description = $3,
            check_type = $4,
            parameters = $5,
            severity = $6,
            enabled = $7,
            updated_at = $8
        WHERE id = $1
        RETURNING id, name, description, check_type, parameters, severity, enabled, created_at, updated_at
        "#,
        id,
        name,
        description,
        check_type,
        parameters,
        severity_str,
        enabled,
        now,
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_check(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM check_definitions WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_check_counts(pool: &PgPool) -> Result<CheckCounts, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE enabled = true) as enabled
        FROM check_definitions
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(CheckCounts {
        total: row.total.unwrap_or(0),
        enabled: row.enabled.unwrap_or(0),
    })
}

#[derive(Debug)]
pub struct CheckCounts {
    pub total: i64,
    pub enabled: i64,
}
