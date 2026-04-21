//! `/api/nights/*` — create/list/edit/close/reopen a poker night.
//!
//! Phase 0 stub. Backend-Core owns this in Phase 1. The close endpoint
//! is the load-bearing piece that wraps settlement in a serializable
//! transaction — see `docs/adr/0002-iou-model.md` and
//! `crate::domain::settlement`.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
