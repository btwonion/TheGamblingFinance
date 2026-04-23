//! Request / response DTOs for the HTTP layer.
//!
//! One module per aggregate, matching the schemas in
//! `docs/contracts/openapi.json`. Keep these modules purely data:
//! validation and I/O live in `routes::*` and `db::*` respectively.

pub mod activity;
pub mod leaderboard;
pub mod night;
pub mod settlement;
pub mod user;
