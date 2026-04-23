//! `/api/users/*` — admin CRUD over player accounts, plus self-update.
//!
//! The `/me` PATCH sub-resource is specifically NOT gated by the admin
//! extractor — any authenticated user may update their own profile.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db,
    dto::user::{
        CreateUserRequest, ResetPasswordRequest, Role, UpdateSelfRequest, UpdateUserRequest, User,
    },
    error::AppError,
    middleware::auth::{AuthedUser, RequireAdmin},
    state::AppState,
    util::password::{hash_password, verify_password},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/users", get(list_users).post(create_user))
        .route("/api/users/me", patch(update_self))
        .route("/api/users/:id", patch(update_user))
        .route("/api/users/:id/reset-password", post(reset_password_admin))
}

async fn list_users(
    _admin: RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = db::users::list_users(&state.pool)
        .await
        .map_err(internal)?;
    Ok(Json(users))
}

async fn create_user(
    _admin: RequireAdmin,
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    if req.email.trim().is_empty() {
        return Err(AppError::BadRequest("email required".into()));
    }
    if req.password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    if req.display_name.trim().is_empty() {
        return Err(AppError::BadRequest("display_name required".into()));
    }

    let hash = hash_password(&req.password)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("hash error: {e}")))?;

    let user = match db::users::create_user(
        &state.pool,
        &req.email,
        &req.display_name,
        &hash,
        req.role,
    )
    .await
    {
        Ok(u) => u,
        Err(e) if is_unique_violation(&e) => {
            return Err(AppError::Conflict("email already in use".into()));
        }
        Err(e) => return Err(internal(e)),
    };

    Ok((StatusCode::CREATED, Json(user)))
}

async fn update_user(
    admin: RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, AppError> {
    // Guard against demoting or disabling the last active admin.
    let target = db::users::find_user_by_id(&state.pool, id)
        .await
        .map_err(internal)?
        .ok_or(AppError::NotFound)?;

    let demoting = match req.role {
        Some(new_role) => new_role != target.role && target.role == Role::Admin,
        None => false,
    };
    let disabling = req.disabled_at.is_some() && target.disabled_at.is_none();
    if (demoting || disabling) && target.role == Role::Admin {
        let active_admins = db::users::count_active_admins(&state.pool)
            .await
            .map_err(internal)?;
        if active_admins <= 1 {
            return Err(AppError::Conflict(
                "refusing to orphan the admin role".into(),
            ));
        }
    }

    // Self-demotion guard (the only admin demoting themselves).
    if demoting && target.id == admin.0.user_id {
        let active_admins = db::users::count_active_admins(&state.pool)
            .await
            .map_err(internal)?;
        if active_admins <= 1 {
            return Err(AppError::Conflict(
                "refusing to demote the last admin".into(),
            ));
        }
    }

    let updated = db::users::update_user(&state.pool, id, &req)
        .await
        .map_err(internal)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(updated))
}

async fn reset_password_admin(
    _admin: RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<StatusCode, AppError> {
    if req.new_password.len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }
    let hash = hash_password(&req.new_password)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("hash error: {e}")))?;
    let ok = db::users::set_password(&state.pool, id, &hash)
        .await
        .map_err(internal)?;
    if !ok {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn update_self(
    caller: AuthedUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateSelfRequest>,
) -> Result<Json<User>, AppError> {
    // Password change path requires current_password verification.
    if let Some(new_password) = req.new_password.as_ref() {
        if new_password.len() < 8 {
            return Err(AppError::BadRequest(
                "password must be at least 8 characters".into(),
            ));
        }
        let current = req.current_password.as_ref().ok_or_else(|| {
            AppError::BadRequest("current_password required to change password".into())
        })?;
        let stored = db::users::find_password_hash(&state.pool, caller.user_id)
            .await
            .map_err(internal)?
            .ok_or(AppError::Unauthorized)?;
        if !verify_password(current, &stored) {
            return Err(AppError::Unauthorized);
        }
        let hash = hash_password(new_password)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("hash error: {e}")))?;
        let ok = db::users::set_password(&state.pool, caller.user_id, &hash)
            .await
            .map_err(internal)?;
        if !ok {
            return Err(AppError::NotFound);
        }
    }

    if let Some(name) = req.display_name.as_deref() {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("display_name cannot be empty".into()));
        }
        db::users::set_display_name(&state.pool, caller.user_id, name)
            .await
            .map_err(internal)?;
    }

    let user = db::users::find_user_by_id(&state.pool, caller.user_id)
        .await
        .map_err(internal)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(user))
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    // `email` is UNIQUE; Postgres SQLSTATE 23505.
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            return code == "23505";
        }
    }
    false
}

fn internal<E: Into<anyhow::Error>>(err: E) -> AppError {
    AppError::Internal(err.into())
}

// Silence the "unused" warning for the `json!` import until we surface
// richer error details payloads in a future iteration.
#[allow(dead_code)]
fn _json_hint() -> serde_json::Value {
    json!({ "placeholder": true })
}
