//! `/api/leaderboard` — lifetime ranking across closed nights.
//!
//! The per-night ranking (`/api/nights/:id/leaderboard`) is handled in
//! `crate::routes::nights` because it shares Path-param plumbing with
//! the other night-scoped routes.

use axum::{extract::State, routing::get, Json, Router};

use crate::{
    db, dto::leaderboard::LeaderboardEntry, error::AppError,
    middleware::auth::AuthedUser, state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/leaderboard", get(lifetime))
}

async fn lifetime(
    _caller: AuthedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<LeaderboardEntry>>, AppError> {
    let rows = db::leaderboard::lifetime(&state.pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(rows))
}
