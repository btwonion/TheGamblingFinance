//! Session-cookie authentication extractors.
//!
//! - `AuthedUser` resolves the `gf_sid` cookie to a live session +
//!   user; 401 if missing/invalid/disabled/revoked/expired.
//! - `RequireAdmin` wraps `AuthedUser` and short-circuits with 403 if
//!   the caller isn't an admin.
//!
//! Both extractors are parameterised over any state `S` that exposes a
//! `PgPool` via `FromRef<S>`. This keeps the middleware module
//! decoupled from the concrete `AppState` struct: any future state
//! type (integration-test harness, admin-only router, etc.) that
//! implements `FromRef<_> for PgPool` will plug in unchanged.
//! `AppState` satisfies the bound via a hand-written impl in
//! `crate::state` (Backend-Core owns that file).

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::auth_sessions,
    dto::user::Role,
    error::AppError,
    util::cookies,
};

/// Authenticated user summary derived from a valid session cookie.
/// This is the shape we hand downstream handlers; the full `User`
/// DTO (with `created_at` etc.) is fetched separately when needed.
#[derive(Debug, Clone)]
pub struct AuthedUser {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: Role,
}

/// Newtype wrapper that asserts the authed user is an admin. Handlers
/// that mutate global state (e.g. create users, create nights) ask for
/// this in their signature; non-admins get 403.
#[derive(Debug, Clone)]
pub struct RequireAdmin(pub AuthedUser);

#[async_trait]
impl<S> FromRequestParts<S> for AuthedUser
where
    S: Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        // 1. Read raw cookie → 401 if missing.
        let raw_token = cookies::read_session_token(&parts.headers)
            .ok_or(AppError::Unauthorized)?;

        // 2. Hash and look up. Any DB error bubbles as 500.
        let hash = cookies::hash_token(&raw_token);
        let found = auth_sessions::find_active_session(&pool, &hash)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

        // 3. Missing, expired, or revoked → the query already filtered
        //    these out, so `None` means "no such live session".
        let session = found.ok_or(AppError::Unauthorized)?;

        // 4. User could have been live-disabled while holding a
        //    session cookie. Treat as 401 (not 403) because the
        //    caller is effectively no longer authenticated.
        if session.disabled_at.is_some() {
            return Err(AppError::Unauthorized);
        }

        // 5. Best-effort last_seen bump. We `await` so the UPDATE
        //    actually runs, but swallow errors so a transient DB
        //    blip on a bookkeeping column never fails the user's
        //    request.
        let _ = auth_sessions::touch_last_seen(&pool, session.session_id).await;

        Ok(AuthedUser {
            user_id: session.user_id,
            email: session.email,
            display_name: session.display_name,
            role: session.role,
        })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthedUser::from_request_parts(parts, state).await?;
        if user.role != Role::Admin {
            return Err(AppError::Forbidden);
        }
        Ok(RequireAdmin(user))
    }
}
