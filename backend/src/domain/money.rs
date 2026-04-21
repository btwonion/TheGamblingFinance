//! `Cents` — integer money with explicit overflow handling.
//!
//! All money in the system is `i64` cents. Floats are never used. This
//! newtype exists to:
//!
//! 1. Make money unmistakable at call sites (`Cents(250)` vs `250`).
//! 2. Force overflow handling to be explicit (`checked_add`).
//! 3. Round-trip cleanly through serde as a plain integer.
//!
//! Phase 0 provides the basic arithmetic; Phase 1 (Backend-Core) adds
//! conversions to/from `Decimal` if we ever need them for reporting.

use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Cents(pub i64);

impl Cents {
    pub const ZERO: Cents = Cents(0);

    #[inline]
    pub fn get(self) -> i64 {
        self.0
    }

    /// Addition that surfaces overflow rather than wrapping.
    #[inline]
    pub fn checked_add(self, other: Cents) -> Option<Cents> {
        self.0.checked_add(other.0).map(Cents)
    }

    /// Subtraction that surfaces overflow rather than wrapping.
    #[inline]
    pub fn checked_sub(self, other: Cents) -> Option<Cents> {
        self.0.checked_sub(other.0).map(Cents)
    }

    /// Multiply cents by an integer (e.g. chips × cents_per_chip).
    /// Returns `None` on overflow.
    #[inline]
    pub fn checked_mul_i64(self, other: i64) -> Option<Cents> {
        self.0.checked_mul(other).map(Cents)
    }
}

impl From<i64> for Cents {
    #[inline]
    fn from(v: i64) -> Self {
        Cents(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_add_detects_overflow() {
        assert!(Cents(i64::MAX).checked_add(Cents(1)).is_none());
        assert_eq!(Cents(2).checked_add(Cents(3)), Some(Cents(5)));
    }

    #[test]
    fn checked_sub_detects_underflow() {
        assert!(Cents(i64::MIN).checked_sub(Cents(1)).is_none());
        assert_eq!(Cents(5).checked_sub(Cents(2)), Some(Cents(3)));
    }

    #[test]
    fn serde_roundtrips_as_integer() {
        let cents = Cents(12_345);
        let s = serde_json::to_string(&cents).unwrap();
        assert_eq!(s, "12345");
        let back: Cents = serde_json::from_str(&s).unwrap();
        assert_eq!(back, cents);
    }
}
