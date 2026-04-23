//! Queries against `trades`.

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::activity::{CreateTradeRequest, Trade};

pub async fn create(
    pool: &PgPool,
    night_id: Uuid,
    req: &CreateTradeRequest,
    created_by: Uuid,
) -> Result<Trade, sqlx::Error> {
    let id = Uuid::now_v7();
    sqlx::query_as::<_, Trade>(
        "INSERT INTO trades \
           (id, night_id, chip_giver_id, chip_receiver_id, chips, amount_cents_owed, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         RETURNING id, night_id, chip_giver_id, chip_receiver_id, chips, amount_cents_owed, created_at",
    )
    .bind(id)
    .bind(night_id)
    .bind(req.chip_giver_id)
    .bind(req.chip_receiver_id)
    .bind(req.chips)
    .bind(req.amount_cents_owed)
    .bind(created_by)
    .fetch_one(pool)
    .await
}

pub async fn delete(
    pool: &PgPool,
    night_id: Uuid,
    trade_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        "DELETE FROM trades \
         WHERE id = $2 AND night_id = $1 \
           AND EXISTS (SELECT 1 FROM nights WHERE id = $1 AND status = 'open')",
    )
    .bind(night_id)
    .bind(trade_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn list_by_night(
    pool: &PgPool,
    night_id: Uuid,
) -> Result<Vec<Trade>, sqlx::Error> {
    sqlx::query_as::<_, Trade>(
        "SELECT id, night_id, chip_giver_id, chip_receiver_id, chips, amount_cents_owed, created_at \
         FROM trades WHERE night_id = $1 \
         ORDER BY created_at ASC, id ASC",
    )
    .bind(night_id)
    .fetch_all(pool)
    .await
}
