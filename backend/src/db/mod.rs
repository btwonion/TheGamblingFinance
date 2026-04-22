//! Database access layer.
//!
//! One submodule per aggregate. All SQL is runtime-checked (no
//! `sqlx::query!` macros) because the sandbox has no live Postgres for
//! `sqlx prepare`. Each module exposes `async fn` repository functions
//! that take a `&PgPool` (for one-shot work) or a
//! `&mut Transaction<'_, Postgres>` (for the close-a-night flow).

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub mod auth_sessions;
pub mod buy_ins;
pub mod cash_outs;
pub mod leaderboard;
pub mod nights;
pub mod settlement;
pub mod trades;
pub mod users;

/// Build a Postgres pool with sensible defaults for a small-team app.
pub async fn connect(url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(5))
        .connect(url)
        .await
}
