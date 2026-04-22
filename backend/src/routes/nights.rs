//! `/api/nights/*` — create/list/edit/close/reopen a poker night.
//!
//! The `close_night` handler is the load-bearing piece: it wraps the
//! settlement computation in a serializable transaction, so two racing
//! admin clicks produce one winner + one 409. See
//! `docs/adr/0002-iou-model.md` for the algorithm and its guarantees.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db,
    dto::{
        activity::{
            BuyIn, CashOut, CreateBuyInRequest, CreateTradeRequest, Trade, UpsertCashOutRequest,
        },
        night::{AddPlayerRequest, CreateNightRequest, NightDetail, NightSummary, UpdateNightRequest},
        settlement::SettlementResponse,
        user::Role,
    },
    error::AppError,
    middleware::auth::{AuthedUser, RequireAdmin},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/nights", get(list_nights).post(create_night))
        .route(
            "/api/nights/:id",
            get(get_night)
                .patch(update_night)
                .delete(delete_night),
        )
        .route(
            "/api/nights/:id/players",
            post(add_player),
        )
        .route(
            "/api/nights/:id/players/:uid",
            delete(remove_player),
        )
        // Activity sub-resources live in their own modules; here we just
        // expose the close/reopen transaction-owning handlers.
        .route("/api/nights/:id/close", post(close_night))
        .route("/api/nights/:id/reopen", post(reopen_night))
        // Activity endpoints. Keeping them all on the same Router keeps
        // `AppState` propagation simple.
        .route(
            "/api/nights/:id/buy-ins",
            post(create_buy_in),
        )
        .route(
            "/api/nights/:id/buy-ins/:buy_in_id",
            delete(delete_buy_in),
        )
        .route(
            "/api/nights/:id/trades",
            post(create_trade),
        )
        .route(
            "/api/nights/:id/trades/:trade_id",
            delete(delete_trade),
        )
        .route(
            "/api/nights/:id/cash-outs/:uid",
            axum::routing::put(upsert_cash_out),
        )
        // Read-only settlement + leaderboard fetches live on the night
        // path too; handlers delegate to the relevant repositories.
        .route(
            "/api/nights/:id/settlement",
            get(get_settlement),
        )
        .route(
            "/api/nights/:id/leaderboard",
            get(get_night_leaderboard),
        )
}

// --- List / create --------------------------------------------------------

async fn list_nights(
    caller: AuthedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<NightSummary>>, AppError> {
    let rows = if caller.role == Role::Admin {
        db::nights::list_for_admin(&state.pool)
            .await
            .map_err(internal)?
    } else {
        db::nights::list_for_player(&state.pool, caller.user_id)
            .await
            .map_err(internal)?
    };
    Ok(Json(rows))
}

async fn create_night(
    admin: RequireAdmin,
    State(state): State<AppState>,
    Json(req): Json<CreateNightRequest>,
) -> Result<(StatusCode, Json<NightDetail>), AppError> {
    if req.title.trim().is_empty() {
        return Err(AppError::BadRequest("title required".into()));
    }
    if req.cents_per_chip <= 0 {
        return Err(AppError::BadRequest(
            "cents_per_chip must be positive".into(),
        ));
    }
    if req.player_ids.is_empty() {
        return Err(AppError::BadRequest("player_ids cannot be empty".into()));
    }
    let detail = db::nights::create(&state.pool, &req, admin.0.user_id)
        .await
        .map_err(internal)?;
    Ok((StatusCode::CREATED, Json(detail)))
}

// --- Read / update / delete ----------------------------------------------

async fn get_night(
    caller: AuthedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NightDetail>, AppError> {
    // Access control: admins see anything; non-admins must be members.
    if caller.role != Role::Admin {
        let member = db::nights::is_member(&state.pool, id, caller.user_id)
            .await
            .map_err(internal)?;
        if !member {
            // Hide existence from non-members → 404, not 403.
            return Err(AppError::NotFound);
        }
    }
    let detail = db::nights::get_detail(&state.pool, id)
        .await
        .map_err(internal)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(detail))
}

async fn update_night(
    _admin: RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNightRequest>,
) -> Result<Json<NightDetail>, AppError> {
    // If the night does not exist → 404; if it's closed → 409.
    match db::nights::is_open(&state.pool, id).await.map_err(internal)? {
        None => return Err(AppError::NotFound),
        Some(false) => {
            return Err(AppError::Conflict("night is closed".into()));
        }
        Some(true) => {}
    }
    let detail = db::nights::update_metadata(&state.pool, id, &req)
        .await
        .map_err(internal)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(detail))
}

async fn delete_night(
    _admin: RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    match db::nights::is_open(&state.pool, id).await.map_err(internal)? {
        None => return Err(AppError::NotFound),
        Some(false) => {
            return Err(AppError::Conflict("cannot delete a closed night".into()));
        }
        Some(true) => {}
    }
    let ok = db::nights::delete(&state.pool, id).await.map_err(internal)?;
    if !ok {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

// --- Membership -----------------------------------------------------------

async fn add_player(
    _admin: RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddPlayerRequest>,
) -> Result<StatusCode, AppError> {
    match db::nights::is_open(&state.pool, id).await.map_err(internal)? {
        None => return Err(AppError::NotFound),
        Some(false) => {
            return Err(AppError::Conflict("night is closed".into()));
        }
        Some(true) => {}
    }
    let added = db::nights::add_player(&state.pool, id, req.user_id)
        .await
        .map_err(|e| {
            if is_fk_violation(&e) {
                AppError::BadRequest("unknown user_id".into())
            } else {
                internal(e)
            }
        })?;
    if !added {
        // Already a member — idempotent success is kinder than 409 here.
        return Ok(StatusCode::NO_CONTENT);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_player(
    _admin: RequireAdmin,
    State(state): State<AppState>,
    Path((id, uid)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    match db::nights::is_open(&state.pool, id).await.map_err(internal)? {
        None => return Err(AppError::NotFound),
        Some(false) => {
            return Err(AppError::Conflict("night is closed".into()));
        }
        Some(true) => {}
    }
    let activity = db::nights::count_player_activity(&state.pool, id, uid)
        .await
        .map_err(internal)?;
    if activity > 0 {
        return Err(AppError::Conflict(
            "player has recorded activity on this night".into(),
        ));
    }
    let removed = db::nights::remove_player(&state.pool, id, uid)
        .await
        .map_err(internal)?;
    if !removed {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

// --- Activity -------------------------------------------------------------

async fn create_buy_in(
    _admin: RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateBuyInRequest>,
) -> Result<(StatusCode, Json<BuyIn>), AppError> {
    if req.amount_cents <= 0 {
        return Err(AppError::BadRequest("amount_cents must be positive".into()));
    }
    if req.chips <= 0 {
        return Err(AppError::BadRequest("chips must be positive".into()));
    }
    match db::nights::is_open(&state.pool, id).await.map_err(internal)? {
        None => return Err(AppError::NotFound),
        Some(false) => {
            return Err(AppError::Conflict("night is closed".into()));
        }
        Some(true) => {}
    }
    let row = db::buy_ins::create(&state.pool, id, &req, _admin.0.user_id)
        .await
        .map_err(|e| {
            if is_fk_violation(&e) {
                AppError::BadRequest("user is not a member of this night".into())
            } else {
                internal(e)
            }
        })?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn delete_buy_in(
    _admin: RequireAdmin,
    State(state): State<AppState>,
    Path((id, buy_in_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    match db::nights::is_open(&state.pool, id).await.map_err(internal)? {
        None => return Err(AppError::NotFound),
        Some(false) => return Err(AppError::Conflict("night is closed".into())),
        Some(true) => {}
    }
    let ok = db::buy_ins::delete(&state.pool, id, buy_in_id)
        .await
        .map_err(internal)?;
    if !ok {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn create_trade(
    admin: RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateTradeRequest>,
) -> Result<(StatusCode, Json<Trade>), AppError> {
    if req.chips <= 0 {
        return Err(AppError::BadRequest("chips must be positive".into()));
    }
    if req.amount_cents_owed <= 0 {
        return Err(AppError::BadRequest(
            "amount_cents_owed must be positive".into(),
        ));
    }
    if req.chip_giver_id == req.chip_receiver_id {
        return Err(AppError::BadRequest("giver and receiver must differ".into()));
    }
    match db::nights::is_open(&state.pool, id).await.map_err(internal)? {
        None => return Err(AppError::NotFound),
        Some(false) => return Err(AppError::Conflict("night is closed".into())),
        Some(true) => {}
    }
    let row = db::trades::create(&state.pool, id, &req, admin.0.user_id)
        .await
        .map_err(|e| {
            if is_fk_violation(&e) {
                AppError::BadRequest("giver or receiver is not a member of this night".into())
            } else {
                internal(e)
            }
        })?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn delete_trade(
    _admin: RequireAdmin,
    State(state): State<AppState>,
    Path((id, trade_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    match db::nights::is_open(&state.pool, id).await.map_err(internal)? {
        None => return Err(AppError::NotFound),
        Some(false) => return Err(AppError::Conflict("night is closed".into())),
        Some(true) => {}
    }
    let ok = db::trades::delete(&state.pool, id, trade_id)
        .await
        .map_err(internal)?;
    if !ok {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn upsert_cash_out(
    admin: RequireAdmin,
    State(state): State<AppState>,
    Path((id, uid)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpsertCashOutRequest>,
) -> Result<Json<CashOut>, AppError> {
    if req.chips < 0 {
        return Err(AppError::BadRequest("chips cannot be negative".into()));
    }
    match db::nights::is_open(&state.pool, id).await.map_err(internal)? {
        None => return Err(AppError::NotFound),
        Some(false) => return Err(AppError::Conflict("night is closed".into())),
        Some(true) => {}
    }
    let row = db::cash_outs::upsert(&state.pool, id, uid, &req, admin.0.user_id)
        .await
        .map_err(|e| {
            if is_fk_violation(&e) {
                AppError::BadRequest("user is not a member of this night".into())
            } else {
                internal(e)
            }
        })?;
    Ok(Json(row))
}

// --- Close / reopen -------------------------------------------------------

async fn close_night(
    _admin: RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SettlementResponse>, AppError> {
    // Serializable isolation so two racing closes collide into one
    // commit + one 40001 (serialization failure) → 409.
    let mut tx = state.pool.begin().await.map_err(internal)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    // 1. Lock the night row and confirm it's open.
    match db::nights::is_open_locked(&mut tx, id)
        .await
        .map_err(internal)?
    {
        None => {
            let _ = tx.rollback().await;
            return Err(AppError::NotFound);
        }
        Some(false) => {
            let _ = tx.rollback().await;
            return Err(AppError::Conflict("night is already closed".into()));
        }
        Some(true) => {}
    }

    // 2. Players vs cash-outs → missing set (reject before running algo).
    let players = db::settlement::player_user_ids(&state.pool, id)
        .await
        .map_err(internal)?;
    let cash_out_users = db::settlement::cash_out_user_ids(&state.pool, id)
        .await
        .map_err(internal)?;
    let cash_out_set: std::collections::HashSet<Uuid> = cash_out_users.iter().copied().collect();
    let missing: Vec<Uuid> = players
        .iter()
        .copied()
        .filter(|p| !cash_out_set.contains(p))
        .collect();
    if !missing.is_empty() {
        let _ = tx.rollback().await;
        return Err(AppError::Conflict(format!(
            "missing cash-outs for {} players: {}",
            missing.len(),
            missing
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    // 3. Load inputs + run settle().
    let input = db::settlement::load_inputs_for_night(&state.pool, id)
        .await
        .map_err(internal)?;
    let output = match crate::domain::settlement::settle(&input) {
        Ok(o) => o,
        Err(crate::domain::settlement::SettleError::UnbalancedLedger { diff_cents }) => {
            let _ = tx.rollback().await;
            return Err(AppError::Conflict(format!(
                "unbalanced ledger: off by {diff_cents} cents"
            )));
        }
        Err(e) => {
            let _ = tx.rollback().await;
            return Err(AppError::Conflict(format!("settlement error: {e}")));
        }
    };

    // 4. Persist inside the same transaction.
    db::settlement::persist_settlement(&mut tx, id, &output)
        .await
        .map_err(internal)?;

    // 5. Commit, translating serialization failure into 409.
    if let Err(e) = tx.commit().await {
        if is_serialization_failure(&e) {
            return Err(AppError::Conflict("close conflict; retry".into()));
        }
        return Err(internal(e));
    }

    // 6. Return a fresh read of the stored settlement (same shape the
    //    GET endpoint produces, ensuring byte-identical responses).
    let resp = db::settlement::load_settlement(&state.pool, id)
        .await
        .map_err(internal)?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("settlement vanished post-commit")))?;
    Ok(Json(resp))
}

async fn reopen_night(
    _admin: RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let ok = db::settlement::delete_settlement(&state.pool, id)
        .await
        .map_err(internal)?;
    if !ok {
        // Could be not found, or open already.
        let status = db::nights::is_open(&state.pool, id)
            .await
            .map_err(internal)?;
        return match status {
            None => Err(AppError::NotFound),
            Some(true) => Err(AppError::Conflict("night is not closed".into())),
            Some(false) => Err(AppError::Internal(anyhow::anyhow!(
                "reopen failed silently on a closed night"
            ))),
        };
    }
    Ok(StatusCode::NO_CONTENT)
}

// --- Read: settlement / leaderboard --------------------------------------

async fn get_settlement(
    caller: AuthedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SettlementResponse>, AppError> {
    // Access: admins always; players must be members.
    if caller.role != Role::Admin {
        let member = db::nights::is_member(&state.pool, id, caller.user_id)
            .await
            .map_err(internal)?;
        if !member {
            return Err(AppError::NotFound);
        }
    }
    let resp = db::settlement::load_settlement(&state.pool, id)
        .await
        .map_err(internal)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(resp))
}

async fn get_night_leaderboard(
    caller: AuthedUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::dto::leaderboard::LeaderboardEntry>>, AppError> {
    if caller.role != Role::Admin {
        let member = db::nights::is_member(&state.pool, id, caller.user_id)
            .await
            .map_err(internal)?;
        if !member {
            return Err(AppError::NotFound);
        }
    }
    // If there's no settlement (open night), this returns an empty list,
    // which is the right answer.
    let rows = db::leaderboard::per_night(&state.pool, id)
        .await
        .map_err(internal)?;
    Ok(Json(rows))
}

// --- helpers --------------------------------------------------------------

fn is_fk_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            return code == "23503";
        }
    }
    false
}

fn is_serialization_failure(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            return code == "40001";
        }
    }
    false
}

fn internal<E: Into<anyhow::Error>>(err: E) -> AppError {
    AppError::Internal(err.into())
}

#[allow(dead_code)]
fn _json_hint() -> serde_json::Value {
    json!({ "placeholder": true })
}
