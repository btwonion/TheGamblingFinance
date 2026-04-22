//! Request middleware: auth extractors, rate-limiters, admin guards.
//!
//! `auth`       — `AuthedUser` and `RequireAdmin` extractors.
//! `rate_limit` — `login_rate_limit_layer` (tower_governor, per-IP).

pub mod auth;
pub mod rate_limit;
