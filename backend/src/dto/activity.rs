//! DTOs for buy-ins, trades, and cash-outs.
//!
//! Money fields stay as plain `i64` cents at the wire/DTO layer; the
//! `Cents` newtype is used inside the settlement algorithm and as
//! runtime invariants, but keeping serde-level cents as `i64` avoids
//! a layer of newtype shims and matches the `openapi.json` shape.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BuyIn {
    pub id: Uuid,
    pub night_id: Uuid,
    pub user_id: Uuid,
    pub amount_cents: i64,
    pub chips: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Trade {
    pub id: Uuid,
    pub night_id: Uuid,
    pub chip_giver_id: Uuid,
    pub chip_receiver_id: Uuid,
    pub chips: i64,
    pub amount_cents_owed: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CashOut {
    pub id: Uuid,
    pub night_id: Uuid,
    pub user_id: Uuid,
    pub chips: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBuyInRequest {
    pub user_id: Uuid,
    pub amount_cents: i64,
    pub chips: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTradeRequest {
    pub chip_giver_id: Uuid,
    pub chip_receiver_id: Uuid,
    pub chips: i64,
    pub amount_cents_owed: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertCashOutRequest {
    pub chips: i64,
}
