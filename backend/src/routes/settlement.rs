//! `/api/nights/:id/settlement` — read the stored settlement.
//!
//! The GET handler lives in `crate::routes::nights`; the compute-and-
//! store side is also there (`POST /nights/:id/close`). The algorithm
//! itself is in `crate::domain::settlement`.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
