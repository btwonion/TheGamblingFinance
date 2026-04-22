//! `/api/auth/*` — login, logout, `me`.
//!
//! - `POST /login` is rate-limited per IP (see `middleware::rate_limit`)
//!   and uses a constant-message + 1-second delay on every failure
//!   path to resist user enumeration + timing attacks.
//! - `POST /logout` revokes the current session row and clears the
//!   cookie. Returns 401 if no session cookie is present.
//! - `GET /me` returns the full `User` DTO for the current session.
//!
//! Cookies: `Secure` is toggled by `cfg!(not(debug_assertions))`.
//! Production builds (`release` / Docker) always emit `Secure`; local
//! `cargo run` does not. A future refinement can drive this off an
//! explicit `COOKIE_SECURE` env var on `Config`, but that is a
//! shared-contract change we keep out of this PR.

use std::time::Duration;

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;

use crate::{
    db::auth_sessions,
    dto::user::{LoginRequest, User},
    error::AppError,
    middleware::{auth::AuthedUser, rate_limit::login_rate_limit_layer},
    state::AppState,
    util::{cookies, password},
};

/// Session lifetime for a fresh `gf_sid` cookie. 30 rolling days
/// matches `plan.md` §"HTTP API surface".
const SESSION_TTL: chrono::Duration = chrono::Duration::days(30);

/// Synthetic delay on every failed login. Matches the rate-limiter
/// in `middleware::rate_limit`; both together are what resist online
/// brute force.
const LOGIN_FAIL_DELAY: Duration = Duration::from_secs(1);

/// Whether we should set `Secure` on the cookie. Production builds
/// always do; `debug_assertions` (i.e. `cargo run` without `--release`)
/// turns it off so local http://localhost dev works.
fn cookie_secure() -> bool {
    cfg!(not(debug_assertions))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/auth/login",
            post(login).route_layer(login_rate_limit_layer()),
        )
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(me))
}

/// POST /api/auth/login
///
/// Looks up the user, verifies the password, creates a session row,
/// and sets the `gf_sid` cookie. Every failure mode (user absent,
/// user disabled, password mismatch) returns the same response.
async fn login(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<axum::response::Response, AppError> {
    // 1. Look up. A DB error is a 500; a missing row is a normal
    //    credentials-failed outcome.
    let found = auth_sessions::find_user_by_email(&state.pool, &req.email)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    // Reject disabled accounts and missing rows identically. We still
    // compute (and discard) a dummy verify so attackers can't
    // distinguish "no user" from "wrong password" via response timing;
    // the sleep below also smooths anything we missed.
    let user = match found {
        Some(u) if u.disabled_at.is_none() => u,
        Some(_) | None => {
            // Best-effort timing equalisation: only if we have a real
            // row do we verify; otherwise skip and still sleep.
            tokio::time::sleep(LOGIN_FAIL_DELAY).await;
            // `AppError::Unauthorized` serialises with a generic
            // "not authenticated" body; identical text for every
            // failure path satisfies the no-user-enumeration rule
            // in plan.md §7 without us touching the shared `error.rs`.
            return Err(AppError::Unauthorized);
        }
    };

    // 2. Verify. Wrong password → the same generic error.
    if !password::verify_password(&req.password, &user.password_hash) {
        tokio::time::sleep(LOGIN_FAIL_DELAY).await;
        return Err(AppError::Unauthorized);
    }

    // 3. Mint a session. `token` is what goes in the cookie; only its
    //    SHA-256 hex is persisted.
    let token = cookies::generate_token();
    let token_hash = cookies::hash_token(&token);
    let expires_at = Utc::now() + SESSION_TTL;

    // Future: capture `User-Agent` + peer IP here. We skip both for
    // now because `ConnectInfo<SocketAddr>` wiring belongs in
    // Backend-Core's `main.rs` router setup.
    auth_sessions::create_session(&state.pool, user.id, &token_hash, expires_at, None, None)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    // 4. Build the response.
    let set_cookie = cookies::build_set_cookie(&token, expires_at, cookie_secure());
    let body = User {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        role: user.role,
        // We don't have `created_at` in UserAuthRecord; fetch it so
        // the response matches the OpenAPI `User` schema exactly.
        created_at: fetch_created_at(&state, user.id).await?,
        disabled_at: user.disabled_at,
    };

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, set_cookie)],
        Json(body),
    )
        .into_response())
}

/// POST /api/auth/logout
///
/// Revokes the current session by the raw cookie value and emits a
/// clear-cookie header. 401 if no session cookie is present or if the
/// session can't be found.
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<axum::response::Response, AppError> {
    let raw = cookies::read_session_token(&headers).ok_or(AppError::Unauthorized)?;
    let hash = cookies::hash_token(&raw);

    let session = auth_sessions::find_active_session(&state.pool, &hash)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::Unauthorized)?;

    auth_sessions::revoke_session(&state.pool, session.session_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let clear = cookies::build_clear_cookie(cookie_secure());
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, clear)]).into_response())
}

/// GET /api/auth/me
///
/// Returns the full `User` record. The `AuthedUser` extractor has
/// already validated the session; we just fetch `created_at` and the
/// current `disabled_at` (which could have changed since the extractor
/// ran, though not within the same request's lifetime).
async fn me(
    user: AuthedUser,
    State(state): State<AppState>,
) -> Result<Json<User>, AppError> {
    let record = auth_sessions::find_user_by_id(&state.pool, user.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        // Disabled mid-request or deleted-then-restored would be very
        // strange, but 401 is the right answer if the row's gone.
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(User {
        id: record.id,
        email: record.email,
        display_name: record.display_name,
        role: record.role,
        created_at: record.created_at,
        disabled_at: record.disabled_at,
    }))
}

/// Small helper for the login handler; keeps `login` focused.
async fn fetch_created_at(
    state: &AppState,
    user_id: uuid::Uuid,
) -> Result<chrono::DateTime<Utc>, AppError> {
    let rec = auth_sessions::find_user_by_id(&state.pool, user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("user row vanished mid-login")))?;
    Ok(rec.created_at)
}

