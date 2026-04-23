//! `gamblingfinance` — binary entry point.
//!
//! Phase 1: construct `AppState` from config + a live `PgPool`, merge
//! every router in `routes::*`, and bind `0.0.0.0:$PORT`.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use gamblingfinance::{config::Config, routes, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = Config::from_env().context("loading config")?;
    tracing::info!(port = config.port, "configuration loaded");

    let pool = gamblingfinance::db::connect(&config.database_url)
        .await
        .context("connecting to database")?;
    let state = AppState {
        pool,
        config: Arc::new(config.clone()),
    };

    // CORS is permissive in Phase 1 dev because the frontend is proxied
    // via Vite in dev and same-origin in prod (behind nginx). Tighten
    // in the DevOps pass.
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app: Router = Router::new()
        .merge(routes::health::router())
        .merge(routes::auth::router())
        .merge(routes::users::router())
        .merge(routes::nights::router())
        .merge(routes::buy_ins::router())
        .merge(routes::trades::router())
        .merge(routes::cash_outs::router())
        .merge(routes::settlement::router())
        .merge(routes::leaderboard::router())
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!(%addr, "binding http listener");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding to {addr}"))?;

    axum::serve(listener, app)
        .await
        .context("axum server terminated")?;

    Ok(())
}

/// Initialise `tracing_subscriber` from the `RUST_LOG` env var, falling
/// back to a sensible default if it is unset.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,gamblingfinance=debug,tower_http=info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();
}
