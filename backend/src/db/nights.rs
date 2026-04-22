//! Queries against `nights` and `night_players`.
//!
//! The `NightSummary` shape includes `player_count`; the repository
//! computes it via subquery so a single round-trip can drive list views.
//! Detail reads (`get_detail`) then fan out to activity tables via the
//! repositories in `buy_ins` / `trades` / `cash_outs`.

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::dto::night::{CreateNightRequest, NightDetail, NightSummary, UpdateNightRequest};
use crate::dto::user::User;

/// Admin view: every night, newest-first.
pub async fn list_for_admin(pool: &PgPool) -> Result<Vec<NightSummary>, sqlx::Error> {
    sqlx::query_as::<_, NightSummary>(
        "SELECT n.id, n.title, n.played_on, n.status, n.cents_per_chip, n.currency, \
                n.opened_at, n.closed_at, \
                COALESCE((SELECT COUNT(*) FROM night_players np WHERE np.night_id = n.id), 0) \
                    AS player_count \
         FROM nights n \
         ORDER BY n.played_on DESC, n.opened_at DESC",
    )
    .fetch_all(pool)
    .await
}

/// Player view: only nights where the caller is a member.
pub async fn list_for_player(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<NightSummary>, sqlx::Error> {
    sqlx::query_as::<_, NightSummary>(
        "SELECT n.id, n.title, n.played_on, n.status, n.cents_per_chip, n.currency, \
                n.opened_at, n.closed_at, \
                COALESCE((SELECT COUNT(*) FROM night_players np2 WHERE np2.night_id = n.id), 0) \
                    AS player_count \
         FROM nights n \
         JOIN night_players np ON np.night_id = n.id \
         WHERE np.user_id = $1 \
         ORDER BY n.played_on DESC, n.opened_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Check whether a user is a member of a night. Used by the detail
/// endpoint to gate non-admin access.
pub async fn is_member(
    pool: &PgPool,
    night_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM night_players WHERE night_id = $1 AND user_id = $2")
            .bind(night_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}

/// Fetch the summary row by id (no activity fan-out).
pub async fn get_summary(
    pool: &PgPool,
    night_id: Uuid,
) -> Result<Option<NightSummary>, sqlx::Error> {
    sqlx::query_as::<_, NightSummary>(
        "SELECT n.id, n.title, n.played_on, n.status, n.cents_per_chip, n.currency, \
                n.opened_at, n.closed_at, \
                COALESCE((SELECT COUNT(*) FROM night_players np WHERE np.night_id = n.id), 0) \
                    AS player_count \
         FROM nights n \
         WHERE n.id = $1",
    )
    .bind(night_id)
    .fetch_optional(pool)
    .await
}

/// Fetch the bare `notes` column for a night (the summary row does not
/// carry notes to keep list payloads small).
pub async fn get_notes(pool: &PgPool, night_id: Uuid) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(Option<String>,)> = sqlx::query_as("SELECT notes FROM nights WHERE id = $1")
        .bind(night_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.and_then(|r| r.0))
}

/// Full night detail including members and activity.
pub async fn get_detail(
    pool: &PgPool,
    night_id: Uuid,
) -> Result<Option<NightDetail>, sqlx::Error> {
    let Some(summary) = get_summary(pool, night_id).await? else {
        return Ok(None);
    };
    let players = list_players(pool, night_id).await?;
    let buy_ins = crate::db::buy_ins::list_by_night(pool, night_id).await?;
    let trades = crate::db::trades::list_by_night(pool, night_id).await?;
    let cash_outs = crate::db::cash_outs::list_by_night(pool, night_id).await?;
    let notes = get_notes(pool, night_id).await?;

    Ok(Some(NightDetail {
        night: summary,
        players,
        buy_ins,
        trades,
        cash_outs,
        notes,
    }))
}

/// Players who are members of a night, joined to `users` for display.
pub async fn list_players(pool: &PgPool, night_id: Uuid) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT u.id, u.email, u.display_name, u.role, u.created_at, u.disabled_at \
         FROM night_players np \
         JOIN users u ON u.id = np.user_id \
         WHERE np.night_id = $1 \
         ORDER BY u.display_name ASC",
    )
    .bind(night_id)
    .fetch_all(pool)
    .await
}

/// Create a night + insert its initial `night_players` rows in a single
/// transaction.
pub async fn create(
    pool: &PgPool,
    req: &CreateNightRequest,
    created_by: Uuid,
) -> Result<NightDetail, sqlx::Error> {
    let night_id = Uuid::now_v7();
    let mut tx = pool.begin().await?;

    sqlx::query(
        "INSERT INTO nights (id, created_by, title, played_on, currency, cents_per_chip, notes) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(night_id)
    .bind(created_by)
    .bind(&req.title)
    .bind(req.played_on)
    .bind(&req.currency)
    .bind(req.cents_per_chip)
    .bind(&req.notes)
    .execute(&mut *tx)
    .await?;

    for uid in &req.player_ids {
        sqlx::query("INSERT INTO night_players (night_id, user_id) VALUES ($1, $2)")
            .bind(night_id)
            .bind(uid)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    // Re-fetch as full detail outside the transaction.
    let detail = get_detail(pool, night_id).await?.ok_or(sqlx::Error::RowNotFound)?;
    Ok(detail)
}

/// Update metadata on an open night. Returns `Ok(None)` if no such
/// night. Returns `Err(sqlx::Error::RowNotFound)` style? — no; we
/// return Ok(None) for either "not found" or "closed", letting the
/// caller reject appropriately after a subsequent status check.
pub async fn update_metadata(
    pool: &PgPool,
    night_id: Uuid,
    req: &UpdateNightRequest,
) -> Result<Option<NightDetail>, sqlx::Error> {
    // Build dynamic SET. Guard with status='open'.
    let mut sets: Vec<&str> = Vec::new();
    if req.title.is_some() {
        sets.push("title = $2");
    }
    if req.played_on.is_some() {
        sets.push("played_on = $3");
    }
    if req.notes.is_some() {
        sets.push("notes = $4");
    }
    if sets.is_empty() {
        // No-op update; fall through to a fresh read so callers get
        // the current row.
        return get_detail(pool, night_id).await;
    }

    let sql = format!(
        "UPDATE nights SET {} WHERE id = $1 AND status = 'open' \
         RETURNING id",
        sets.join(", ")
    );
    let updated: Option<(Uuid,)> = sqlx::query_as(&sql)
        .bind(night_id)
        .bind(req.title.as_deref().unwrap_or(""))
        .bind(req.played_on.unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()))
        .bind(req.notes.as_deref().unwrap_or(""))
        .fetch_optional(pool)
        .await?;

    if updated.is_none() {
        return Ok(None);
    }
    get_detail(pool, night_id).await
}

/// Delete an open night. Cascade handles membership + activity rows.
pub async fn delete(pool: &PgPool, night_id: Uuid) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM nights WHERE id = $1 AND status = 'open'")
        .bind(night_id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

/// Add a player to an open night. Returns true iff the row was inserted.
pub async fn add_player(
    pool: &PgPool,
    night_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    // Guard with status='open' via a WHERE-clause INSERT ... SELECT.
    let res = sqlx::query(
        "INSERT INTO night_players (night_id, user_id) \
         SELECT $1, $2 FROM nights WHERE id = $1 AND status = 'open' \
         ON CONFLICT DO NOTHING",
    )
    .bind(night_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Count a player's activity on a night (buy-ins + trades on either
/// side + cash-outs). Used by `remove_player` to refuse cleanly.
pub async fn count_player_activity(
    pool: &PgPool,
    night_id: Uuid,
    user_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT \
           (SELECT COUNT(*) FROM buy_ins WHERE night_id = $1 AND user_id = $2) + \
           (SELECT COUNT(*) FROM trades  WHERE night_id = $1 AND (chip_giver_id = $2 OR chip_receiver_id = $2)) + \
           (SELECT COUNT(*) FROM cash_outs WHERE night_id = $1 AND user_id = $2)",
    )
    .bind(night_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(n)
}

/// Remove a player. Fails if the night is closed or the player has
/// activity; the caller must check `count_player_activity` first.
pub async fn remove_player(
    pool: &PgPool,
    night_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        "DELETE FROM night_players \
         WHERE night_id = $1 AND user_id = $2 \
           AND EXISTS (SELECT 1 FROM nights WHERE id = $1 AND status = 'open')",
    )
    .bind(night_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Check whether a night is in the `open` state.
pub async fn is_open(pool: &PgPool, night_id: Uuid) -> Result<Option<bool>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as("SELECT status FROM nights WHERE id = $1")
        .bind(night_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0 == "open"))
}

/// Used inside the close transaction — checks status while holding the
/// row lock. Returns `Some(true)` = open, `Some(false)` = closed, `None`
/// = no such night.
pub async fn is_open_locked(
    tx: &mut Transaction<'_, Postgres>,
    night_id: Uuid,
) -> Result<Option<bool>, sqlx::Error> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT status FROM nights WHERE id = $1 FOR UPDATE")
            .bind(night_id)
            .fetch_optional(&mut **tx)
            .await?;
    Ok(row.map(|r| r.0 == "open"))
}
