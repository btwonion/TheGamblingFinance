//! Pure domain logic — no I/O, no HTTP, no DB.
//!
//! - `money`      — `Cents` newtype with checked arithmetic.
//! - `settlement` — the greedy min-cashflow solver. Phase 0 locks the
//!                  input/output signature; Backend-Core fills in the
//!                  body in Phase 1.

pub mod money;
pub mod settlement;
