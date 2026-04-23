//! Queries against `users` (for login lookups) and `auth_sessions`.
//!
//! All SQL is runtime-checked via `sqlx::query_as` / `sqlx::query` —
//! the project has no live Postgres in the sandbox so `sqlx::query!`
//! macros would refuse to compile without an offline `.sqlx/` cache.
//! The returned row types derive `FromRow` so column order in the
//! SELECT must match the struct field order or sqlx will return a
//! column-mismatch error at runtime.

use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::user::Role;

/// Flat row returned by the login-time user lookup. We intentionally
/// include `password_hash` here (unlike the public `User` DTO) because
/// this is the only path that needs it.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserAuthRecord {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: Role,
    pub password_hash: String,
    pub disabled_at: Option<DateTime<Utc>>,
}

/// Row returned when resolving a session cookie to a live session +
/// its owning user. `session_id` is included so callers can bump
/// `last_seen_at` or revoke the row on logout without a second round
/// trip.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionLookup {
    pub session_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: Role,
    pub disabled_at: Option<DateTime<Utc>>,
}

/// Full user row by id; used by `GET /auth/me` and other endpoints
/// that need the canonical `User` DTO fields (not just the extractor
/// summary).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
}

/// Look up a user by email (case-insensitive via the `citext` column).
/// Returns `Ok(None)` when no row matches. The login handler MUST treat
/// "no user", "wrong password", and "disabled" as indistinguishable at
/// the wire level to avoid user enumeration.
pub async fn find_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<UserAuthRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserAuthRecord>(
        "SELECT id, email, display_name, role, password_hash, disabled_at \
         FROM users \
         WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

/// Fetch the public-facing user record by id. Used by the `/auth/me`
/// handler to produce a full `User` DTO (needs `created_at`).
pub async fn find_user_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserRecord>(
        "SELECT id, email, display_name, role, created_at, disabled_at \
         FROM users \
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Insert a new row into `auth_sessions`. Returns the generated
/// UUID v7. The caller is responsible for hashing the raw cookie token
/// (see `util::cookies::hash_token`) and choosing an expiry.
pub async fn create_session(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
    user_agent: Option<&str>,
    ip: Option<IpNetwork>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO auth_sessions \
           (id, user_id, token_hash, expires_at, user_agent, ip_addr) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .bind(user_agent)
    .bind(ip)
    .execute(pool)
    .await?;
    Ok(id)
}

/// Resolve a hashed cookie token to a live session + its user.
/// Returns `Ok(None)` if no row matches, the row is revoked, or the
/// row has expired — the extractor treats all three as 401.
pub async fn find_active_session(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<SessionLookup>, sqlx::Error> {
    sqlx::query_as::<_, SessionLookup>(
        "SELECT \
             s.id            AS session_id, \
             s.expires_at, \
             s.revoked_at, \
             u.id            AS user_id, \
             u.email, \
             u.display_name, \
             u.role, \
             u.disabled_at \
         FROM auth_sessions s \
         JOIN users u ON u.id = s.user_id \
         WHERE s.token_hash = $1 \
           AND s.revoked_at IS NULL \
           AND s.expires_at > now()",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await
}

/// Bump `last_seen_at` to `now()`. Called fire-and-forget from the
/// `AuthedUser` extractor so the user list can show a rough activity
/// time; we intentionally swallow the `Result` at the call site.
pub async fn touch_last_seen(pool: &PgPool, session_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE auth_sessions SET last_seen_at = now() WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Mark the session revoked. Idempotent — rerunning is a no-op that
/// overwrites `revoked_at` with a fresh timestamp, which is fine.
pub async fn revoke_session(pool: &PgPool, session_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE auth_sessions SET revoked_at = now() WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}
