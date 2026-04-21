//! HTTP route modules.
//!
//! Each module exposes a `router()` function that returns an
//! `axum::Router<AppState>` (or `Router<()>` for stateless routes like
//! `health`). `main.rs` merges them under the `/api` prefix.
//!
//! Phase 0 only implements `health`. The other modules are declared
//! here with empty `router()` functions so Phase 1 agents can plug in
//! handlers without reshaping the module tree — and so that imports
//! from `tests/` remain stable across phases.

pub mod auth;
pub mod buy_ins;
pub mod cash_outs;
pub mod health;
pub mod leaderboard;
pub mod nights;
pub mod settlement;
pub mod trades;
pub mod users;
