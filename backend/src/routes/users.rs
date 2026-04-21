//! `/api/users/*` — admin CRUD over player accounts, plus self-update.
//!
//! Phase 0 stub. Backend-Core owns this in Phase 1.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
