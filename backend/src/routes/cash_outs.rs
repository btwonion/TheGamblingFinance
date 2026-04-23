//! `/api/nights/:id/cash-outs/:uid` — upsert a player's end-of-night cash-out.
//!
//! The handler lives in `crate::routes::nights` alongside the other
//! night-scoped resources.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
