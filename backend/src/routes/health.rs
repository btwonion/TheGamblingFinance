//! `GET /api/health` — liveness probe.
//!
//! The only route wired in Phase 0. Returns `{status, git_sha}` so
//! Docker/Compose healthchecks and humans both have something to curl.
//!
//! `git_sha` is read from the `GF_GIT_SHA` build-time env var (set by
//! the Dockerfile); absent, we report `"unknown"`.

use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
pub struct Health {
    pub status: &'static str,
    pub git_sha: &'static str,
}

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().route("/api/health", get(handler))
}

async fn handler() -> Json<Health> {
    Json(Health {
        status: "ok",
        git_sha: option_env!("GF_GIT_SHA").unwrap_or("unknown"),
    })
}
