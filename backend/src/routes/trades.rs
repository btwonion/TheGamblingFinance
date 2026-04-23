//! `/api/nights/:id/trades` — IOU trades between players.
//!
//! The handlers live in `crate::routes::nights`. See the note there.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
