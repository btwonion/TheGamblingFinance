//! Uniform error type and response envelope for the HTTP layer.
//!
//! The wire shape is `{ "error": { "code", "message", "details"? } }`
//! from `plan.md` §"HTTP API surface". Handlers return
//! `Result<T, AppError>` and the `IntoResponse` impl below formats the
//! envelope and picks an HTTP status code.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::Value as Json_;

/// Errors surfaced at the HTTP boundary. Phase 1 agents extend this
/// enum with domain-specific variants (e.g.
/// `SettlementUnbalanced { diff_cents: i64 }`).
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("not authenticated")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("not found")]
    NotFound,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("too many requests")]
    RateLimited,

    #[error("validation error")]
    Validation { details: Json_ },

    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
            AppError::Validation { .. } => (StatusCode::UNPROCESSABLE_ENTITY, "validation_error"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    code: &'a str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Json_>,
}

#[derive(Serialize)]
struct ErrorEnvelope<'a> {
    error: ErrorBody<'a>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log internals before we drop them into a 500.
        if let AppError::Internal(err) = &self {
            tracing::error!(error = ?err, "internal error");
        }

        let (status, code) = self.status_and_code();
        let details = match &self {
            AppError::Validation { details } => Some(details.clone()),
            _ => None,
        };
        let body = ErrorEnvelope {
            error: ErrorBody {
                code,
                message: self.to_string(),
                details,
            },
        };
        (status, Json(body)).into_response()
    }
}
