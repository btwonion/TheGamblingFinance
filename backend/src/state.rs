//! Application state shared across handlers.
//!
//! Phase 0 does not construct an `AppState` yet (the only route is
//! `/api/health` and it needs nothing). Phase 1 agents build it in
//! `main.rs` and pass it into the router via `.with_state(...)`.

use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
}

// Phase 0 gap patched in Phase 1 (Backend-Core): the auth extractors in
// `crate::middleware::auth` are parameterised over any state `S` where
// `PgPool: FromRef<S>`. Without this impl, `AuthedUser` / `RequireAdmin`
// cannot extract the pool from `AppState`. Adding the trivial forwarder
// here keeps middleware decoupled from concrete state fields.
impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}
