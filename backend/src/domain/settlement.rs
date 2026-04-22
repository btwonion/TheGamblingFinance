//! Settlement algorithm.
//!
//! **This file locks the shared contract between Backend-Core and the
//! frontend parity test.** The input/output types and the
//! `pub fn settle(...)` signature may not change without a handshake
//! — see `docs/adr/0002-iou-model.md` and `docs/agents.md`.
//!
//! Body implementation (Phase 1):
//!
//! 1. compute per-player `net_cents`,
//! 2. assert `sum(net_cents) == 0` or return `UnbalancedLedger`,
//! 3. run the greedy creditors/debtors pairing,
//! 4. emit deterministic transfers (tie-break on `user_id` byte order).
//!
//! The algorithm is versioned via `ALGO_VERSION`; bump it any time the
//! output shape or ordering changes so historical `settlements.algo_version`
//! rows remain interpretable.

use std::collections::HashMap;

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementOutput {
    pub algo_version: i16,
    pub balances: Vec<Balance>,
    pub transfers: Vec<Transfer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance {
    pub user_id: Uuid,
    pub net_cents: Cents,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
/// Step 1 — per-player net cents (checked arithmetic throughout):
///
/// ```text
/// cash_value(p)   = cash_outs[p].chips * cents_per_chip
/// buy_in_total(p) = sum of p's buy_ins.amount_cents
/// iou_received(p) = sum of trade.cents_owed WHERE chip_giver == p
/// iou_owed(p)     = sum of trade.cents_owed WHERE chip_receiver == p
/// net(p) = cash_value(p) - buy_in_total(p) + iou_received(p) - iou_owed(p)
/// ```
///
/// Step 2 — require `sum(net) == 0`, else `UnbalancedLedger`.
/// Step 3 — greedy min-cashflow with user_id byte-order tie-breaking.
pub fn settle(input: &SettlementInput) -> Result<SettlementOutput, SettleError> {
    // Deduplicate & maintain set semantics while keeping the sorted
    // output order stable. A small HashMap<UserId, Cents> is fine here
    // — N is the number of players (typically <= 10).
    let mut nets: HashMap<Uuid, Cents> = HashMap::with_capacity(input.players.len());
    for pid in &input.players {
        // First insert pins membership; duplicate player_ids are a
        // repository bug — we silently collapse here.
        nets.entry(*pid).or_insert(Cents::ZERO);
    }

    // Validate trade memberships first — cheaper than computing
    // partial nets only to discard them.
    for trade in &input.trades {
        if !nets.contains_key(&trade.chip_giver_id) {
            return Err(SettleError::UnknownPlayer(trade.chip_giver_id));
        }
        if !nets.contains_key(&trade.chip_receiver_id) {
            return Err(SettleError::UnknownPlayer(trade.chip_receiver_id));
        }
    }
    for b in &input.buy_ins {
        if !nets.contains_key(&b.user_id) {
            return Err(SettleError::UnknownPlayer(b.user_id));
        }
    }
    for c in &input.cash_outs {
        if !nets.contains_key(&c.user_id) {
            return Err(SettleError::UnknownPlayer(c.user_id));
        }
    }

    // cash_value = chips * cents_per_chip  (added to net)
    for c in &input.cash_outs {
        let value = Cents(c.chips)
            .checked_mul_i64(input.cents_per_chip)
            .ok_or(SettleError::Overflow)?;
        let entry = nets.get_mut(&c.user_id).expect("checked above");
        *entry = entry.checked_add(value).ok_or(SettleError::Overflow)?;
    }

    // buy_in_total subtracted from net.
    for b in &input.buy_ins {
        let entry = nets.get_mut(&b.user_id).expect("checked above");
        *entry = entry.checked_sub(b.amount_cents).ok_or(SettleError::Overflow)?;
    }

    // iou_received: chip_giver (the one the IOU is owed TO) receives the cents.
    // iou_owed    : chip_receiver (the one who took chips) owes the cents.
    for t in &input.trades {
        let giver_entry = nets.get_mut(&t.chip_giver_id).expect("checked above");
        *giver_entry = giver_entry
            .checked_add(t.amount_cents_owed)
            .ok_or(SettleError::Overflow)?;
        let receiver_entry = nets.get_mut(&t.chip_receiver_id).expect("checked above");
        *receiver_entry = receiver_entry
            .checked_sub(t.amount_cents_owed)
            .ok_or(SettleError::Overflow)?;
    }

    // Step 2 — sum must be zero. Overflow-safe summation: because every
    // input value is already in i64-cents and the individual nets have
    // stayed in i64 range via checked arithmetic, the sum of `nets`
    // values fits in i128 trivially.
    let diff_cents: i128 = nets.values().map(|c| c.get() as i128).sum();
    if diff_cents != 0 {
        // i128 → i64 is safe when |sum| fits, which it always does for
        // realistic home-poker inputs. Saturate rather than panic.
        let clamped = diff_cents.clamp(i64::MIN as i128, i64::MAX as i128) as i64;
        return Err(SettleError::UnbalancedLedger { diff_cents: clamped });
    }

    // Build the stable-ordered balances: by user_id byte order.
    let mut balances: Vec<Balance> = nets
        .iter()
        .map(|(id, cents)| Balance {
            user_id: *id,
            net_cents: *cents,
        })
        .collect();
    balances.sort_by(|a, b| a.user_id.as_bytes().cmp(b.user_id.as_bytes()));

    // Step 3 — greedy min-cashflow.
    // creditors: net > 0, sorted DESC by net then ASC by user_id.
    // debtors:   net < 0 (stored as magnitude), sorted DESC by magnitude then ASC by user_id.
    let mut creditors: Vec<(Uuid, i64)> = balances
        .iter()
        .filter(|b| b.net_cents.get() > 0)
        .map(|b| (b.user_id, b.net_cents.get()))
        .collect();
    let mut debtors: Vec<(Uuid, i64)> = balances
        .iter()
        .filter(|b| b.net_cents.get() < 0)
        // Store magnitude so the inner loop works with positives only.
        .map(|b| (b.user_id, -b.net_cents.get()))
        .collect();

    creditors.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.as_bytes().cmp(b.0.as_bytes())));
    debtors.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.as_bytes().cmp(b.0.as_bytes())));

    let mut transfers: Vec<Transfer> = Vec::new();
    let mut seq: i32 = 0;
    let mut ci = 0usize;
    let mut di = 0usize;
    while ci < creditors.len() && di < debtors.len() {
        let (creditor_id, credit) = creditors[ci];
        let (debtor_id, debit) = debtors[di];
        let pay = credit.min(debit);
        debug_assert!(pay > 0, "zero transfers should not be emitted");

        transfers.push(Transfer {
            from_user_id: debtor_id,
            to_user_id: creditor_id,
            amount_cents: Cents(pay),
            seq,
        });
        seq = seq.checked_add(1).ok_or(SettleError::Overflow)?;

        creditors[ci].1 -= pay;
        debtors[di].1 -= pay;
        if creditors[ci].1 == 0 {
            ci += 1;
        }
        if debtors[di].1 == 0 {
            di += 1;
        }
    }

    Ok(SettlementOutput {
        algo_version: ALGO_VERSION,
        balances,
        transfers,
    })
}

// ---------------------------------------------------------------------
// Unit tests — table-driven, DB-free, cover named scenarios.
// ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn u(byte: u8) -> Uuid {
        Uuid::from_bytes([byte; 16])
    }

    #[test]
    fn single_player_no_transfers() {
        let p = u(1);
        let input = SettlementInput {
            night_id: u(0),
            cents_per_chip: 10,
            players: vec![p],
            buy_ins: vec![BuyInInput {
                user_id: p,
                amount_cents: Cents(5000),
                chips: 500,
            }],
            trades: vec![],
            cash_outs: vec![CashOutInput {
                user_id: p,
                chips: 500,
            }],
        };
        let out = settle(&input).unwrap();
        assert_eq!(out.balances.len(), 1);
        assert_eq!(out.balances[0].net_cents, Cents(0));
        assert!(out.transfers.is_empty());
        assert_eq!(out.algo_version, ALGO_VERSION);
    }

    #[test]
    fn two_players_one_transfer() {
        // A buys in 100 cents, ends with 0 chips → -100.
        // B buys in 0 cents, ends with 10 chips * 10 = 100 → +100.
        // B must pay A? No — A lost, so A pays B is wrong. Re-check:
        // net(A) = cash_value - buy_in = 0 - 100 = -100  (A pays)
        // net(B) = cash_value - buy_in = 100 - 0 = +100 (B receives)
        // Transfer: from A to B, amount 100.
        let a = u(1);
        let b = u(2);
        let input = SettlementInput {
            night_id: u(0),
            cents_per_chip: 10,
            players: vec![a, b],
            buy_ins: vec![BuyInInput {
                user_id: a,
                amount_cents: Cents(100),
                chips: 10,
            }],
            trades: vec![],
            cash_outs: vec![
                CashOutInput { user_id: a, chips: 0 },
                CashOutInput { user_id: b, chips: 10 },
            ],
        };
        let out = settle(&input).unwrap();
        assert_eq!(out.transfers.len(), 1);
        let tr = &out.transfers[0];
        assert_eq!(tr.from_user_id, a);
        assert_eq!(tr.to_user_id, b);
        assert_eq!(tr.amount_cents, Cents(100));
        assert_eq!(tr.seq, 0);
        // Balances sum to zero.
        let sum: i64 = out.balances.iter().map(|b| b.net_cents.get()).sum();
        assert_eq!(sum, 0);
    }

    #[test]
    fn three_players_with_iou() {
        // cents_per_chip = 10.
        // A buys in 200 cents (20 chips). Cashes out 10 chips  -> cash_value 100.
        // B buys in 200 cents (20 chips). Cashes out 40 chips  -> cash_value 400.
        // C buys in 200 cents (20 chips). Cashes out 10 chips  -> cash_value 100.
        // Trade: A gives B 10 chips for 100 cents owed (B owes A 100).
        //    -> iou_received(A) = 100; iou_owed(B) = 100.
        // net(A) = 100 - 200 + 100 - 0 = 0
        // net(B) = 400 - 200 + 0 - 100 = 100
        // net(C) = 100 - 200 + 0 - 0 = -100
        // Expected: C pays B 100.
        let a = u(1);
        let b = u(2);
        let c = u(3);
        let input = SettlementInput {
            night_id: u(0),
            cents_per_chip: 10,
            players: vec![a, b, c],
            buy_ins: vec![
                BuyInInput { user_id: a, amount_cents: Cents(200), chips: 20 },
                BuyInInput { user_id: b, amount_cents: Cents(200), chips: 20 },
                BuyInInput { user_id: c, amount_cents: Cents(200), chips: 20 },
            ],
            trades: vec![TradeInput {
                chip_giver_id: a,
                chip_receiver_id: b,
                chips: 10,
                amount_cents_owed: Cents(100),
            }],
            cash_outs: vec![
                CashOutInput { user_id: a, chips: 10 },
                CashOutInput { user_id: b, chips: 40 },
                CashOutInput { user_id: c, chips: 10 },
            ],
        };
        let out = settle(&input).unwrap();
        assert_eq!(out.transfers.len(), 1);
        let tr = &out.transfers[0];
        assert_eq!(tr.from_user_id, c);
        assert_eq!(tr.to_user_id, b);
        assert_eq!(tr.amount_cents, Cents(100));
        // Balances summed.
        let sum: i64 = out.balances.iter().map(|bal| bal.net_cents.get()).sum();
        assert_eq!(sum, 0);
        // Balance ordering is user_id byte order.
        assert_eq!(out.balances[0].user_id, a);
        assert_eq!(out.balances[1].user_id, b);
        assert_eq!(out.balances[2].user_id, c);
    }

    #[test]
    fn unbalanced_returns_error() {
        // A buys 100, cashes 0  → -100
        // B buys 0,   cashes 5 * 10 = 50 → +50
        // Sum = -50.
        let a = u(1);
        let b = u(2);
        let input = SettlementInput {
            night_id: u(0),
            cents_per_chip: 10,
            players: vec![a, b],
            buy_ins: vec![BuyInInput {
                user_id: a,
                amount_cents: Cents(100),
                chips: 10,
            }],
            trades: vec![],
            cash_outs: vec![
                CashOutInput { user_id: a, chips: 0 },
                CashOutInput { user_id: b, chips: 5 },
            ],
        };
        match settle(&input) {
            Err(SettleError::UnbalancedLedger { diff_cents }) => {
                assert_eq!(diff_cents, -50);
            }
            other => panic!("expected UnbalancedLedger, got {other:?}"),
        }
    }

    #[test]
    fn unknown_player_in_buy_in() {
        let a = u(1);
        let stranger = u(99);
        let input = SettlementInput {
            night_id: u(0),
            cents_per_chip: 1,
            players: vec![a],
            buy_ins: vec![BuyInInput {
                user_id: stranger,
                amount_cents: Cents(10),
                chips: 10,
            }],
            trades: vec![],
            cash_outs: vec![],
        };
        assert_eq!(settle(&input), Err(SettleError::UnknownPlayer(stranger)));
    }

    #[test]
    fn overflow_in_cash_value_returns_overflow() {
        let a = u(1);
        // chips * cents_per_chip = i64::MAX * 2 → overflow.
        let input = SettlementInput {
            night_id: u(0),
            cents_per_chip: i64::MAX,
            players: vec![a],
            buy_ins: vec![],
            trades: vec![],
            cash_outs: vec![CashOutInput { user_id: a, chips: 2 }],
        };
        assert_eq!(settle(&input), Err(SettleError::Overflow));
    }

    #[test]
    fn tie_breaking_by_user_id_bytes_is_deterministic() {
        // Two creditors with equal net (+50, +50) and two debtors with
        // equal net (-50, -50). Expect creditors/debtors sorted by id
        // bytes ascending on ties, so the resulting transfer sequence
        // pairs up in that order.
        let a = u(0x01);
        let b = u(0x02);
        let c = u(0x03);
        let d = u(0x04);
        // A and B cashed out enough to be +50 each.
        // C and D bought in but got nothing → -50 each.
        // cents_per_chip=1; no trades.
        let input = SettlementInput {
            night_id: u(0),
            cents_per_chip: 1,
            players: vec![d, a, c, b], // intentionally out of order
            buy_ins: vec![
                BuyInInput { user_id: c, amount_cents: Cents(50), chips: 50 },
                BuyInInput { user_id: d, amount_cents: Cents(50), chips: 50 },
            ],
            trades: vec![],
            cash_outs: vec![
                CashOutInput { user_id: a, chips: 50 },
                CashOutInput { user_id: b, chips: 50 },
                CashOutInput { user_id: c, chips: 0 },
                CashOutInput { user_id: d, chips: 0 },
            ],
        };
        let out = settle(&input).unwrap();
        // Two transfers, both $50; debtors sorted (C then D), creditors
        // sorted (A then B).
        assert_eq!(out.transfers.len(), 2);
        assert_eq!(out.transfers[0].from_user_id, c);
        assert_eq!(out.transfers[0].to_user_id, a);
        assert_eq!(out.transfers[0].seq, 0);
        assert_eq!(out.transfers[1].from_user_id, d);
        assert_eq!(out.transfers[1].to_user_id, b);
        assert_eq!(out.transfers[1].seq, 1);
        // Determinism: re-run must produce identical output.
        let again = settle(&input).unwrap();
        assert_eq!(out.transfers.len(), again.transfers.len());
        for (x, y) in out.transfers.iter().zip(again.transfers.iter()) {
            assert_eq!(x.from_user_id, y.from_user_id);
            assert_eq!(x.to_user_id, y.to_user_id);
            assert_eq!(x.amount_cents, y.amount_cents);
            assert_eq!(x.seq, y.seq);
        }
    }

    #[test]
    fn empty_players_no_transfers() {
        let input = SettlementInput {
            night_id: u(0),
            cents_per_chip: 10,
            players: vec![],
            buy_ins: vec![],
            trades: vec![],
            cash_outs: vec![],
        };
        let out = settle(&input).unwrap();
        assert!(out.balances.is_empty());
        assert!(out.transfers.is_empty());
    }
}
