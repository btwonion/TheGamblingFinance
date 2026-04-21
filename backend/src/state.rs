//! Application state shared across handlers.
//!
//! Phase 0 does not construct an `AppState` yet (the only route is
//! `/api/health` and it needs nothing). Phase 1 agents build it in
//! `main.rs` and pass it into the router via `.with_state(...)`.

use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
}
