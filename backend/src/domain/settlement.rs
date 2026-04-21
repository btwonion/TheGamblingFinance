//! Settlement algorithm.
//!
//! **This file locks the shared contract between Backend-Core and the
//! frontend parity test.** The input/output types and the
//! `pub fn settle(...)` signature may not change without a handshake
//! — see `docs/adr/0002-iou-model.md` and `docs/agents.md`.
//!
//! Phase 0 implements only the signature and types. Backend-Core fills
//! in the body in Phase 1:
//!
//! 1. compute per-player `net_cents`,
//! 2. assert `sum(net_cents) == 0` or return `UnbalancedLedger`,
//! 3. run the greedy creditors/debtors pairing,
//! 4. emit deterministic transfers (tie-break on `user_id` byte order).
//!
//! The algorithm is versioned via `ALGO_VERSION`; bump it any time the
//! output shape or ordering changes so historical `settlements.algo_version`
//! rows remain interpretable.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::money::Cents;

/// Bumped whenever the algorithm's behavior changes. Stored per
/// settlement row so old results remain traceable.
pub const ALGO_VERSION: i16 = 1;

/// All the inputs the algorithm needs. The caller (close endpoint)
/// assembles this from DB rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementInput {
    pub night_id: Uuid,
    pub cents_per_chip: i64,
    pub players: Vec<Uuid>,
    pub buy_ins: Vec<BuyInInput>,
    pub trades: Vec<TradeInput>,
    pub cash_outs: Vec<CashOutInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyInInput {
    pub user_id: Uuid,
    pub amount_cents: Cents,
    pub chips: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInput {
    pub chip_giver_id: Uuid,
    pub chip_receiver_id: Uuid,
    pub chips: i64,
    pub amount_cents_owed: Cents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashOutInput {
    pub user_id: Uuid,
    pub chips: i64,
}

/// Algorithm output. `balances` and `transfers` are in deterministic
/// order; consumers may rely on it for stable UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementOutput {
    pub algo_version: i16,
    pub balances: Vec<Balance>,
    pub transfers: Vec<Transfer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub user_id: Uuid,
    pub net_cents: Cents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub amount_cents: Cents,
    pub seq: i32,
}

/// Reasons `settle` can refuse to produce a result.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SettleError {
    /// Net cents across players don't sum to zero — typically means a
    /// cash-out is missing or a buy-in/trade is wrong. The UI shows
    /// this as "Books off by X € — check cash-outs".
    #[error("unbalanced ledger: off by {diff_cents} cents")]
    UnbalancedLedger { diff_cents: i64 },

    /// An input references a user who is not in `players`.
    #[error("unknown player {0} referenced by activity row")]
    UnknownPlayer(Uuid),

    /// Arithmetic overflow on i64 cents. Realistically unreachable for
    /// home-poker sums, but we surface rather than wrap.
    #[error("arithmetic overflow while computing nets")]
    Overflow,
}

/// Compute the settlement. **Signature is locked in Phase 0.**
///
/// Backend-Core implements the body in Phase 1.
pub fn settle(_input: &SettlementInput) -> Result<SettlementOutput, SettleError> {
    // Intentionally left unimplemented: Phase 0 only locks the shape.
    // Returning an error makes accidental use obvious at runtime while
    // still allowing callers to pattern-match on the result.
    unimplemented!("settlement::settle is implemented in Phase 1 by Backend-Core")
}
