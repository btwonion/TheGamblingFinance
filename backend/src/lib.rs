//! `gamblingfinance` — library entry point.
//!
//! The crate is primarily a binary (see `src/main.rs`); this library
//! target exists so integration tests in `tests/` can reach internal
//! modules without duplicating code. Phase 0 keeps these modules as
//! empty scaffolds. Phase 1 agents (Backend-Core, Backend-Auth) will
//! fill them in.

pub mod config;
pub mod db;
pub mod domain;
pub mod dto;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod state;
pub mod util;
