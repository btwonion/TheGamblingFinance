//! `/api/leaderboard` and `/api/nights/:id/leaderboard`.
//!
//! Phase 0 stub. Backend-Core owns this in Phase 1. Lifetime figures
//! are computed by summing `settlement_balances.net_cents` across all
//! closed nights — see the index on `settlement_balances(user_id)`.

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
}
