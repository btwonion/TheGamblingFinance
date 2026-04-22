//! Queries against the `users` table.
//!
//! All SQL is runtime-checked via `sqlx::query_as` / `sqlx::query` —
//! the project has no live Postgres in the sandbox so `sqlx::query!`
//! macros would refuse to compile without an offline `.sqlx/` cache.
//!
//! The login-side queries + `UserAuthRecord` live in
//! `crate::db::auth_sessions` (Backend-Auth). This module only covers
//! admin-facing CRUD over the public `User` DTO.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::user::{Role, UpdateUserRequest, User};

/// List every user, enabled or disabled, ordered by display_name.
pub async fn list_users(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, display_name, role, created_at, disabled_at \
         FROM users \
         ORDER BY display_name ASC",
    )
    .fetch_all(pool)
    .await
}

/// Lookup a single user by id.
pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, display_name, role, created_at, disabled_at \
         FROM users \
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Insert a new row in `users`. The caller must have already hashed
/// the password (see `crate::util::password::hash_password`) — we
/// refuse to take a plaintext here so that lane isn't violated.
pub async fn create_user(
    pool: &PgPool,
    email: &str,
    display_name: &str,
    password_hash: &str,
    role: Role,
) -> Result<User, sqlx::Error> {
    let id = Uuid::now_v7();
    sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, display_name, password_hash, role) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, email, display_name, role, created_at, disabled_at",
    )
    .bind(id)
    .bind(email)
    .bind(display_name)
    .bind(password_hash)
    .bind(role)
    .fetch_one(pool)
    .await
}

/// Partial update. Any field left as `None` in `req` is skipped.
///
/// `disabled_at` handling: if `req.clear_disabled_at == true`, set to
/// NULL; else if `req.disabled_at == Some(ts)`, set to `ts`; else leave
/// untouched. This sidesteps JSON's "absent vs null" ambiguity without
/// pulling `serde_with` into the dep tree.
pub async fn update_user(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateUserRequest,
) -> Result<Option<User>, sqlx::Error> {
    // Build the SET clause dynamically. Every update bumps updated_at.
    // We keep the SQL simple and deterministic rather than trying to
    // compose predicates with sea-query or similar.
    let mut sets: Vec<&str> = Vec::new();
    if req.display_name.is_some() {
        sets.push("display_name = $2");
    }
    if req.role.is_some() {
        sets.push("role = $3");
    }
    if req.clear_disabled_at {
        sets.push("disabled_at = NULL");
    } else if req.disabled_at.is_some() {
        sets.push("disabled_at = $4");
    }
    sets.push("updated_at = now()");

    let sql = format!(
        "UPDATE users SET {} WHERE id = $1 \
         RETURNING id, email, display_name, role, created_at, disabled_at",
        sets.join(", ")
    );

    let mut q = sqlx::query_as::<_, User>(&sql).bind(id);
    // Bind parameters in the order referenced by the SET clauses ($2..).
    q = q.bind(req.display_name.as_deref().unwrap_or(""));
    q = q.bind(req.role.unwrap_or(Role::Player));
    q = q.bind::<Option<DateTime<Utc>>>(req.disabled_at);

    q.fetch_optional(pool).await
}

/// Replace the password hash for a user.
pub async fn set_password(
    pool: &PgPool,
    id: Uuid,
    password_hash: &str,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("UPDATE users SET password_hash = $2, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(password_hash)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

/// Number of admins who are still enabled. Used by the update-user
/// handler to refuse operations that would orphan the admin role
/// (disable/demote the last one).
pub async fn count_active_admins(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE role = 'admin' AND disabled_at IS NULL",
    )
    .fetch_one(pool)
    .await?;
    Ok(n)
}

/// Fetch the stored password hash for a user. Used by the self-update
/// handler to verify the caller's `current_password`.
pub async fn find_password_hash(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as("SELECT password_hash FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0))
}

/// Update display_name on the caller's own row.
pub async fn set_display_name(
    pool: &PgPool,
    id: Uuid,
    display_name: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "UPDATE users SET display_name = $2, updated_at = now() WHERE id = $1 \
         RETURNING id, email, display_name, role, created_at, disabled_at",
    )
    .bind(id)
    .bind(display_name)
    .fetch_optional(pool)
    .await
}
