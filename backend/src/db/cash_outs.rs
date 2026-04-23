//! Queries against `cash_outs`.
//!
//! PUT semantics on the HTTP side map to ON CONFLICT DO UPDATE here:
//! one row per `(night_id, user_id)`.

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::activity::{CashOut, UpsertCashOutRequest};

/// Upsert the cash-out for a player. Returns the current row.
pub async fn upsert(
    pool: &PgPool,
    night_id: Uuid,
    user_id: Uuid,
    req: &UpsertCashOutRequest,
    created_by: Uuid,
) -> Result<CashOut, sqlx::Error> {
    let id = Uuid::now_v7();
    sqlx::query_as::<_, CashOut>(
        "INSERT INTO cash_outs (id, night_id, user_id, chips, created_by) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (night_id, user_id) DO UPDATE SET chips = EXCLUDED.chips \
         RETURNING id, night_id, user_id, chips, created_at",
    )
    .bind(id)
    .bind(night_id)
    .bind(user_id)
    .bind(req.chips)
    .bind(created_by)
    .fetch_one(pool)
    .await
}

pub async fn list_by_night(
    pool: &PgPool,
    night_id: Uuid,
) -> Result<Vec<CashOut>, sqlx::Error> {
    sqlx::query_as::<_, CashOut>(
        "SELECT id, night_id, user_id, chips, created_at \
         FROM cash_outs WHERE night_id = $1 \
         ORDER BY created_at ASC, id ASC",
    )
    .bind(night_id)
    .fetch_all(pool)
    .await
}
