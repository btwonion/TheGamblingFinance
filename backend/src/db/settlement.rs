//! Repository for the settlement aggregate (balances + transfers +
//! the `settlements` marker row).
//!
//! `persist_settlement` is designed to be called inside the
//! serializable transaction opened by the close handler; it takes a
//! `&mut Transaction` so callers stay in control of commit/rollback.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::money::Cents;
use crate::domain::settlement::{
    BuyInInput, CashOutInput, SettlementInput, SettlementOutput, TradeInput,
};
use crate::dto::settlement::{SettlementBalance, SettlementResponse, SettlementTransfer};

/// Load everything the settlement algorithm needs for a single night.
pub async fn load_inputs_for_night(
    pool: &PgPool,
    night_id: Uuid,
) -> Result<SettlementInput, sqlx::Error> {
    // cents_per_chip first — if no such night, bail with RowNotFound.
    let (cents_per_chip,): (i32,) =
        sqlx::query_as("SELECT cents_per_chip FROM nights WHERE id = $1")
            .bind(night_id)
            .fetch_one(pool)
            .await?;

    // Players (just ids).
    let players: Vec<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM night_players WHERE night_id = $1")
            .bind(night_id)
            .fetch_all(pool)
            .await?;
    let players: Vec<Uuid> = players.into_iter().map(|(x,)| x).collect();

    // Activity rows — same queries the DTO endpoints use, but we only
    // need the fields the algorithm consumes.
    let buy_ins_rows: Vec<(Uuid, i64, i64)> = sqlx::query_as(
        "SELECT user_id, amount_cents, chips FROM buy_ins WHERE night_id = $1",
    )
    .bind(night_id)
    .fetch_all(pool)
    .await?;
    let buy_ins: Vec<BuyInInput> = buy_ins_rows
        .into_iter()
        .map(|(uid, amount_cents, chips)| BuyInInput {
            user_id: uid,
            amount_cents: Cents(amount_cents),
            chips,
        })
        .collect();

    let trade_rows: Vec<(Uuid, Uuid, i64, i64)> = sqlx::query_as(
        "SELECT chip_giver_id, chip_receiver_id, chips, amount_cents_owed \
         FROM trades WHERE night_id = $1",
    )
    .bind(night_id)
    .fetch_all(pool)
    .await?;
    let trades: Vec<TradeInput> = trade_rows
        .into_iter()
        .map(|(giver, receiver, chips, owed)| TradeInput {
            chip_giver_id: giver,
            chip_receiver_id: receiver,
            chips,
            amount_cents_owed: Cents(owed),
        })
        .collect();

    let cash_out_rows: Vec<(Uuid, i64)> =
        sqlx::query_as("SELECT user_id, chips FROM cash_outs WHERE night_id = $1")
            .bind(night_id)
            .fetch_all(pool)
            .await?;
    let cash_outs: Vec<CashOutInput> = cash_out_rows
        .into_iter()
        .map(|(uid, chips)| CashOutInput { user_id: uid, chips })
        .collect();

    Ok(SettlementInput {
        night_id,
        cents_per_chip: cents_per_chip as i64,
        players,
        buy_ins,
        trades,
        cash_outs,
    })
}

/// Fetch the cash-out user_ids for a night, so the handler can compare
/// against players to detect missing cash-outs before running settle().
pub async fn cash_out_user_ids(
    pool: &PgPool,
    night_id: Uuid,
) -> Result<Vec<Uuid>, sqlx::Error> {
    let rows: Vec<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM cash_outs WHERE night_id = $1")
            .bind(night_id)
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(x,)| x).collect())
}

/// Player user_ids for a night.
pub async fn player_user_ids(
    pool: &PgPool,
    night_id: Uuid,
) -> Result<Vec<Uuid>, sqlx::Error> {
    let rows: Vec<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM night_players WHERE night_id = $1")
            .bind(night_id)
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(x,)| x).collect())
}

/// Insert the settlement aggregate and flip the night to `closed`.
/// Runs entirely inside the caller's transaction.
pub async fn persist_settlement(
    tx: &mut Transaction<'_, Postgres>,
    night_id: Uuid,
    output: &SettlementOutput,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO settlements (night_id, algo_version) VALUES ($1, $2)")
        .bind(night_id)
        .bind(output.algo_version)
        .execute(&mut **tx)
        .await?;

    for bal in &output.balances {
        sqlx::query(
            "INSERT INTO settlement_balances (night_id, user_id, net_cents) \
             VALUES ($1, $2, $3)",
        )
        .bind(night_id)
        .bind(bal.user_id)
        .bind(bal.net_cents.get())
        .execute(&mut **tx)
        .await?;
    }

    for transfer in &output.transfers {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO settlement_transfers \
               (id, night_id, from_user_id, to_user_id, amount_cents, seq) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(night_id)
        .bind(transfer.from_user_id)
        .bind(transfer.to_user_id)
        .bind(transfer.amount_cents.get())
        .bind(transfer.seq)
        .execute(&mut **tx)
        .await?;
    }

    sqlx::query("UPDATE nights SET status = 'closed', closed_at = now() WHERE id = $1")
        .bind(night_id)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

/// Read the stored settlement back out for the GET endpoint.
pub async fn load_settlement(
    pool: &PgPool,
    night_id: Uuid,
) -> Result<Option<SettlementResponse>, sqlx::Error> {
    let row: Option<(DateTime<Utc>, i16)> =
        sqlx::query_as("SELECT computed_at, algo_version FROM settlements WHERE night_id = $1")
            .bind(night_id)
            .fetch_optional(pool)
            .await?;
    let Some((computed_at, algo_version)) = row else {
        return Ok(None);
    };

    let balances = sqlx::query_as::<_, SettlementBalance>(
        "SELECT user_id, net_cents FROM settlement_balances WHERE night_id = $1 \
         ORDER BY user_id ASC",
    )
    .bind(night_id)
    .fetch_all(pool)
    .await?;

    let transfers = sqlx::query_as::<_, SettlementTransfer>(
        "SELECT id, from_user_id, to_user_id, amount_cents, seq \
         FROM settlement_transfers WHERE night_id = $1 \
         ORDER BY seq ASC",
    )
    .bind(night_id)
    .fetch_all(pool)
    .await?;

    Ok(Some(SettlementResponse {
        night_id,
        computed_at,
        algo_version,
        balances,
        transfers,
    }))
}

/// Tear down the stored settlement and flip the night back to `open`.
/// Single transaction; callers should not do anything else inside.
pub async fn delete_settlement(pool: &PgPool, night_id: Uuid) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Require that the night is currently closed. If the row does not
    // exist or isn't closed, we don't want to silently clear transfer
    // rows for an open night (there shouldn't be any, but be defensive).
    let locked: Option<(String,)> =
        sqlx::query_as("SELECT status FROM nights WHERE id = $1 FOR UPDATE")
            .bind(night_id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some((status,)) = locked else {
        return Ok(false);
    };
    if status != "closed" {
        return Ok(false);
    }

    sqlx::query("DELETE FROM settlement_transfers WHERE night_id = $1")
        .bind(night_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM settlement_balances WHERE night_id = $1")
        .bind(night_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM settlements WHERE night_id = $1")
        .bind(night_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE nights SET status = 'open', closed_at = NULL WHERE id = $1")
        .bind(night_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(true)
}
