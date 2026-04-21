//! `/api/nights/:id/trades` — IOU trades between players.
//!
//! Phase 0 stub. Backend-Core owns this in Phase 1.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
