//! DTOs for `/api/leaderboard` and `/api/nights/:id/leaderboard`.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaderboardEntry {
    pub user_id: Uuid,
    pub display_name: String,
    pub net_cents: i64,
    pub nights_played: i64,
}
