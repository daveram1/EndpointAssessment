use chrono::{DateTime, Utc};
use common::{AdminRole, AdminUser};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
    role: AdminRole,
) -> Result<AdminUser, sqlx::Error> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let role_str = role.to_string();

    let row = sqlx::query_as!(
        UserRow,
        r#"
        INSERT INTO admin_users (id, username, password_hash, role, created_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, username, password_hash, role, created_at
        "#,
        id,
        username,
        password_hash,
        role_str,
        now,
    )
    .fetch_one(pool)
    .await?;

    Ok(row.into_user())
}

pub async fn get_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AdminUser>, sqlx::Error> {
    let row = sqlx::query_as!(
        UserRow,
        r#"
        SELECT id, username, password_hash, role, created_at
        FROM admin_users WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.into_user()))
}

pub async fn get_user_by_username(pool: &PgPool, username: &str) -> Result<Option<AdminUser>, sqlx::Error> {
    let row = sqlx::query_as!(
        UserRow,
        r#"
        SELECT id, username, password_hash, role, created_at
        FROM admin_users WHERE username = $1
        "#,
        username
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.into_user()))
}

pub async fn list_users(pool: &PgPool) -> Result<Vec<AdminUser>, sqlx::Error> {
    let rows = sqlx::query_as!(
        UserRow,
        r#"
        SELECT id, username, password_hash, role, created_at
        FROM admin_users ORDER BY username
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into_user()).collect())
}

pub async fn delete_user(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM admin_users WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn user_count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!("SELECT COUNT(*) as count FROM admin_users")
        .fetch_one(pool)
        .await?;

    Ok(row.count.unwrap_or(0))
}

struct UserRow {
    id: Uuid,
    username: String,
    password_hash: String,
    role: Option<String>,
    created_at: Option<DateTime<Utc>>,
}

impl UserRow {
    fn into_user(self) -> AdminUser {
        let role = self
            .role
            .and_then(|r| r.parse().ok())
            .unwrap_or(AdminRole::Viewer);

        AdminUser {
            id: self.id,
            username: self.username,
            password_hash: self.password_hash,
            role,
            created_at: self.created_at.unwrap_or_else(Utc::now),
        }
    }
}
