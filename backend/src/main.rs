//! `gamblingfinance` — binary entry point.
//!
//! Phase 0 scope: wire up tracing, load config, bind an Axum server on
//! `$PORT`, and serve a single `/api/health` endpoint so the Docker
//! image has something real to expose. Phase 1 agents extend the
//! router via `routes::build()`.

use std::net::SocketAddr;

use anyhow::Context;
use axum::Router;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use gamblingfinance::{config::Config, routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    // Phase 0: `Config::from_env` only requires PORT. DATABASE_URL and
    // SESSION_SECRET become mandatory in Phase 1 when Backend-Core and
    // Backend-Auth start using them.
    let config = Config::from_env().context("loading config")?;
    tracing::info!(port = config.port, "configuration loaded");

    // CORS is permissive in Phase 0 because frontend is proxied via
    // Vite in dev and same-origin in prod (behind nginx in DevOps
    // phase). Tighten when we serve cross-origin.
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app: Router = Router::new()
        .merge(routes::health::router())
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
