//! Request middleware: auth extractors, rate-limiters, admin guards.
//!
//! Phase 0 is empty. Backend-Auth adds `AuthedUser` and `RequireAdmin`
//! extractors plus a `tower_governor`-based rate limiter in Phase 1.
