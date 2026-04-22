//! `/api/nights/:id/buy-ins` — record/delete buy-ins.
//!
//! The actual handlers live in `crate::routes::nights` so that all
//! night-scoped resources share one router and consistent Path extractor
//! parameter naming. This module is kept as an empty stub so the
//! `routes::build()` merge list in `main.rs` stays declarative.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
