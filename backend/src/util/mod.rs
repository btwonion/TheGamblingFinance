//! Miscellaneous utilities.
//!
//! `password` — argon2id PHC hash + verify.
//! `cookies`  — `gf_sid` token generation, hashing, and Set-Cookie
//! helpers for the session-cookie auth scheme.

pub mod cookies;
pub mod password;
