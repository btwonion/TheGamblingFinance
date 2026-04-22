//! DTOs for `/api/users` and `/api/auth/*`.
//!
//! `User` and `Role` are part of the **shared contract** with
//! Backend-Auth: `middleware::auth` imports `Role` from here. Do not
//! change field names, types, or the serde/sqlx attributes without a
//! handshake.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Role on the global account level. Stored in `users.role` as plain
/// TEXT with a `CHECK` constraint; the sqlx `Type` mapping below is
/// what makes `SELECT role FROM users` decode directly into a `Role`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum Role {
    Admin,
    Player,
}

/// Public user DTO. Matches `User` in `docs/contracts/openapi.json`;
/// `password_hash` is deliberately excluded.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub display_name: String,
    pub password: String,
    pub role: Role,
}

/// Admin edit payload. Every field is optional so that clients can
/// send partial updates. `disabled_at: None` in the request is
/// interpreted as "leave unchanged" (we do not attempt to distinguish
/// "unset" from "explicit null"; the admin UI can re-enable a user by
/// PATCH'ing another field while sending `disabled_at: null` — see
/// the handler for the pragmatic contract).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub role: Option<Role>,
    /// Use `serde_with`'s double-option if a future requirement needs
    /// to distinguish "absent" from "null"; for Phase 1 we accept
    /// `null` to clear and omission to leave unchanged via a separate
    /// `clear_disabled_at` helper flag implicit in handler logic.
    pub disabled_at: Option<DateTime<Utc>>,
    /// Explicit boolean to clear the `disabled_at` flag (re-enable).
    /// This works around the unset-vs-null ambiguity without pulling
    /// in `serde_with`.
    #[serde(default)]
    pub clear_disabled_at: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateSelfRequest {
    pub display_name: Option<String>,
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}
