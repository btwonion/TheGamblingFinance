//! DTOs for `/api/nights/*` (metadata only; activity is in
//! `dto::activity`, settlement in `dto::settlement`).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dto::activity::{BuyIn, CashOut, Trade};
use crate::dto::user::User;

/// Poker-night status. Stored as TEXT with a CHECK constraint; the
/// sqlx Type mapping matches the migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum NightStatus {
    Open,
    Closed,
}

/// Compact row used in list endpoints. `player_count` is computed via
/// subquery at fetch time.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NightSummary {
    pub id: Uuid,
    pub title: String,
    pub played_on: NaiveDate,
    pub status: NightStatus,
    pub cents_per_chip: i32,
    pub currency: String,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub player_count: i64,
}

/// Everything the Night Detail view needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightDetail {
    pub night: NightSummary,
    pub players: Vec<User>,
    pub buy_ins: Vec<BuyIn>,
    pub trades: Vec<Trade>,
    pub cash_outs: Vec<CashOut>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateNightRequest {
    pub title: String,
    pub played_on: NaiveDate,
    pub cents_per_chip: i32,
    #[serde(default = "default_currency")]
    pub currency: String,
    pub player_ids: Vec<Uuid>,
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_currency() -> String {
    "EUR".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateNightRequest {
    pub title: Option<String>,
    pub played_on: Option<NaiveDate>,
    /// Flat `Option<String>`: `Some("text")` sets, `None` leaves
    /// unchanged; to clear notes send an empty string.
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddPlayerRequest {
    pub user_id: Uuid,
}
