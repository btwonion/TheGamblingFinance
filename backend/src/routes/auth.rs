//! `/api/auth/*` — login, logout, `me`.
//!
//! Phase 0 stub. Backend-Auth owns this in Phase 1 and adds the
//! `POST /login`, `POST /logout`, `GET /me` handlers plus the
//! rate-limiter middleware on login.

use axum::Router;

use crate::state::AppState;

/// Returns the auth router. Empty in Phase 0.
pub fn router() -> Router<AppState> {
    Router::new()
}
