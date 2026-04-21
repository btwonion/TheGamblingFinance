//! Runtime configuration loaded from environment variables.
//!
//! Phase 0 only needs `PORT` to bind the HTTP listener. The other
//! fields are declared now so the shape is stable for Phase 1 agents;
//! `database_url` and `session_secret` default to empty strings in
//! Phase 0 and are validated by the layers that actually use them.

use serde::Deserialize;

/// All runtime configuration, parsed from the process environment.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// HTTP listener port.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Postgres connection string. Required from Phase 1 onwards.
    #[serde(default)]
    pub database_url: String,

    /// Random 32-byte hex used for session cookie signing /
    /// server-side HMAC. Backend-Auth validates length.
    #[serde(default)]
    pub session_secret: String,
}

fn default_port() -> u16 {
    8080
}

impl Config {
    /// Load from `std::env::vars()` via `envy`. Unknown variables are
    /// ignored so the process inherits arbitrary operator-set env.
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Self>()
    }
}
