//! Database access layer.
//!
//! One submodule per aggregate (users, nights, buy_ins, …) in Phase 1.
//! Phase 0 only exposes a shared `connect()` helper so `main.rs` (and,
//! later, migrations) can build a pool from a `DATABASE_URL` string.

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// Build a Postgres pool with sensible defaults for a small-team app.
///
/// Kept minimal intentionally: Phase 1 agents can add richer options
/// (statement cache, SSL mode) as needs arise.
pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(5))
        .connect(url)
        .await
}
