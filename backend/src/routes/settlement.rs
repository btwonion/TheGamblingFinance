//! `/api/nights/:id/settlement` — read the stored settlement.
//!
//! Phase 0 stub. Backend-Core owns this in Phase 1. The compute-and-
//! store side lives on `POST /nights/:id/close` (see `nights.rs`); this
//! module only serves the read side. The algorithm itself is in
//! `crate::domain::settlement`.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
