//! Queries against `buy_ins`.

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::activity::{BuyIn, CreateBuyInRequest};

/// Insert a new buy-in. The composite FK `(night_id, user_id)` already
/// enforces that the player is a member of the night.
pub async fn create(
    pool: &PgPool,
    night_id: Uuid,
    req: &CreateBuyInRequest,
    created_by: Uuid,
) -> Result<BuyIn, sqlx::Error> {
    let id = Uuid::now_v7();
    sqlx::query_as::<_, BuyIn>(
        "INSERT INTO buy_ins (id, night_id, user_id, amount_cents, chips, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id, night_id, user_id, amount_cents, chips, created_at",
    )
    .bind(id)
    .bind(night_id)
    .bind(req.user_id)
    .bind(req.amount_cents)
    .bind(req.chips)
    .bind(created_by)
    .fetch_one(pool)
    .await
}

/// Delete a single buy-in on an open night.
pub async fn delete(
    pool: &PgPool,
    night_id: Uuid,
    buy_in_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        "DELETE FROM buy_ins \
         WHERE id = $2 AND night_id = $1 \
           AND EXISTS (SELECT 1 FROM nights WHERE id = $1 AND status = 'open')",
    )
    .bind(night_id)
    .bind(buy_in_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Every buy-in on a night, oldest first.
pub async fn list_by_night(
    pool: &PgPool,
    night_id: Uuid,
) -> Result<Vec<BuyIn>, sqlx::Error> {
    sqlx::query_as::<_, BuyIn>(
        "SELECT id, night_id, user_id, amount_cents, chips, created_at \
         FROM buy_ins WHERE night_id = $1 \
         ORDER BY created_at ASC, id ASC",
    )
    .bind(night_id)
    .fetch_all(pool)
    .await
}
