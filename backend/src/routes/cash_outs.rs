//! `/api/nights/:id/cash-outs/:uid` — upsert a player's end-of-night cash-out.
//!
//! Phase 0 stub. Backend-Core owns this in Phase 1.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
