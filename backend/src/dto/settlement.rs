//! Wire-level settlement DTOs.
//!
//! These mirror the `SettlementResponse` schema in
//! `docs/contracts/openapi.json`. Note: the **algorithm's** input/output
//! types live in `crate::domain::settlement` (locked signature); these
//! are what the HTTP layer ships to the frontend.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SettlementBalance {
    pub user_id: Uuid,
    pub net_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SettlementTransfer {
    pub id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub amount_cents: i64,
    pub seq: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementResponse {
    pub night_id: Uuid,
    pub computed_at: DateTime<Utc>,
    pub algo_version: i16,
    pub balances: Vec<SettlementBalance>,
    pub transfers: Vec<SettlementTransfer>,
}
