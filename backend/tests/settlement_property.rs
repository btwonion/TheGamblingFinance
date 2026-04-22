//! Property-based tests for `settle()`.
//!
//! These are pure — no DB, no network. They generate random but
//! structurally-valid settlement inputs and assert the algorithm's
//! invariants:
//!
//! 1. Sum of balances is zero.
//! 2. Number of transfers is at most N-1 where N is the number of
//!    non-zero balances.
//! 3. Transfer totals "in" and "out" per user equal that user's |net|.
//! 4. Same input → byte-identical output (determinism).
//!
//! We also assert that deliberately-unbalanced inputs yield
//! `UnbalancedLedger`.

use std::collections::HashMap;

use proptest::prelude::*;
use uuid::Uuid;

use gamblingfinance::domain::money::Cents;
use gamblingfinance::domain::settlement::{
    settle, BuyInInput, CashOutInput, SettleError, SettlementInput, TradeInput,
};

/// Produce a deterministic UUID from a u8 so shrunken failures are
/// human-readable. The full 16 bytes are `id` repeated.
fn uid(id: u8) -> Uuid {
    Uuid::from_bytes([id; 16])
}

/// Build a balanced input: every player buys in some chips, cash-outs
/// sum to the total chips distributed, cents_per_chip is fixed. No
/// trades here — a separate test exercises the trade path.
fn balanced_no_trades_strategy() -> impl Strategy<Value = SettlementInput> {
    // 1..=7 players, cents_per_chip in a small positive range.
    (1u8..=7, 1i64..=10)
        .prop_flat_map(|(n, cpc)| {
            let ids: Vec<Uuid> = (1..=n).map(uid).collect();
            // Per-player buy-in chips, 0..=20 each.
            let buy_chips = prop::collection::vec(0i64..=20, ids.len());
            // Per-player cash-out chips, 0..=20 each, adjusted below.
            let cash_chips = prop::collection::vec(0i64..=20, ids.len());
            // Per-player buy-in cents (must be > 0 if chips > 0).
            // We pick a random price-per-chip-during-buy-in 1..=10.
            let buy_prices = prop::collection::vec(1i64..=10, ids.len());
            (Just(ids), Just(cpc), buy_chips, cash_chips, buy_prices)
        })
        .prop_map(|(ids, cpc, buy_chips, mut cash_chips, buy_prices)| {
            // Make the cash_value sum == sum(buy_ins.amount_cents)
            // so that sum(net) = 0. Concretely:
            //
            //   sum(cash_chips) * cpc == sum(buy_chips) * (avg buy_price?)
            //
            // Simplest: set buy_amounts = buy_chips[i] * cpc (i.e. each
            // player buys in at exactly cpc per chip), and ensure
            // sum(cash_chips) == sum(buy_chips).

            // Rebalance cash_chips so its sum == sum(buy_chips).
            let bsum: i64 = buy_chips.iter().sum();
            let csum: i64 = cash_chips.iter().sum();
            if bsum != csum {
                // Walk cash_chips in order and increase/decrease to reach target.
                let diff = bsum - csum;
                if diff > 0 {
                    // add `diff` chips to the first slot (no cap issue — proptest will shrink).
                    cash_chips[0] = cash_chips[0].saturating_add(diff);
                } else {
                    let mut need = -diff;
                    for c in cash_chips.iter_mut() {
                        let take = (*c).min(need);
                        *c -= take;
                        need -= take;
                        if need == 0 {
                            break;
                        }
                    }
                }
            }

            // Drop buy-ins with chips==0 (amount_cents>0 is a CHECK).
            let buy_ins: Vec<BuyInInput> = buy_chips
                .iter()
                .zip(buy_prices.iter())
                .zip(ids.iter())
                .filter_map(|((chips, _price), uid_)| {
                    if *chips == 0 {
                        None
                    } else {
                        Some(BuyInInput {
                            user_id: *uid_,
                            amount_cents: Cents(chips * cpc),
                            chips: *chips,
                        })
                    }
                })
                .collect();

            let cash_outs: Vec<CashOutInput> = cash_chips
                .iter()
                .zip(ids.iter())
                .map(|(chips, uid_)| CashOutInput {
                    user_id: *uid_,
                    chips: *chips,
                })
                .collect();

            SettlementInput {
                night_id: uid(0),
                cents_per_chip: cpc,
                players: ids,
                buy_ins,
                trades: vec![],
                cash_outs,
            }
        })
}

/// As above but also inject one trade between two distinct players,
/// whose amount_cents_owed washes out because the trade is a pure
/// transfer between two players in the same balanced system.
fn balanced_with_one_trade_strategy() -> impl Strategy<Value = SettlementInput> {
    balanced_no_trades_strategy().prop_flat_map(|base| {
        if base.players.len() < 2 {
            return Just(base).boxed();
        }
        let max_idx = base.players.len() - 1;
        (0usize..=max_idx, 0usize..=max_idx, 1i64..=100)
            .prop_map(move |(i, j, owed)| {
                let mut out = base.clone();
                let i = i % out.players.len();
                let mut k = j % out.players.len();
                if k == i {
                    k = (k + 1) % out.players.len();
                }
                out.trades.push(TradeInput {
                    chip_giver_id: out.players[i],
                    chip_receiver_id: out.players[k],
                    chips: 1,
                    amount_cents_owed: Cents(owed),
                });
                out
            })
            .boxed()
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    #[test]
    fn balanced_no_trades_always_settles(input in balanced_no_trades_strategy()) {
        let out = settle(&input).expect("balanced input should produce Ok");
        // Invariant 1.
        let sum: i64 = out.balances.iter().map(|b| b.net_cents.get()).sum();
        prop_assert_eq!(sum, 0);

        // Invariant 2: at most N-1 transfers where N is non-zero players.
        let non_zero = out.balances.iter().filter(|b| b.net_cents.get() != 0).count();
        if non_zero == 0 {
            prop_assert_eq!(out.transfers.len(), 0);
        } else {
            prop_assert!(out.transfers.len() <= non_zero.saturating_sub(1));
        }

        // Invariant 3: transfers reconcile balances.
        let mut expected: HashMap<Uuid, i64> =
            out.balances.iter().map(|b| (b.user_id, b.net_cents.get())).collect();
        for t in &out.transfers {
            *expected.entry(t.from_user_id).or_insert(0) += t.amount_cents.get();
            *expected.entry(t.to_user_id).or_insert(0) -= t.amount_cents.get();
        }
        for (_uid, remainder) in &expected {
            prop_assert_eq!(*remainder, 0);
        }
    }

    #[test]
    fn balanced_with_trade_always_settles(input in balanced_with_one_trade_strategy()) {
        let out = settle(&input).expect("balanced input should produce Ok");
        let sum: i64 = out.balances.iter().map(|b| b.net_cents.get()).sum();
        prop_assert_eq!(sum, 0);
    }

    #[test]
    fn unbalanced_inputs_return_unbalanced_error(
        input in balanced_no_trades_strategy(),
        drift in 1i64..=1_000
    ) {
        // Perturb by adding a bogus buy-in to one player.
        let mut perturbed = input;
        if perturbed.players.is_empty() {
            return Ok(());
        }
        perturbed.buy_ins.push(BuyInInput {
            user_id: perturbed.players[0],
            amount_cents: Cents(drift),
            chips: 1,
        });
        match settle(&perturbed) {
            Err(SettleError::UnbalancedLedger { diff_cents }) => {
                // The drift should show up in diff_cents.
                prop_assert!(diff_cents != 0);
            }
            Ok(_) => prop_assert!(false, "expected UnbalancedLedger"),
            Err(other) => prop_assert!(false, "expected UnbalancedLedger, got {other:?}"),
        }
    }

    #[test]
    fn determinism_same_input_same_output(input in balanced_no_trades_strategy()) {
        let a = settle(&input).expect("first call");
        let b = settle(&input).expect("second call");
        prop_assert_eq!(a.algo_version, b.algo_version);
        prop_assert_eq!(a.balances.len(), b.balances.len());
        for (x, y) in a.balances.iter().zip(b.balances.iter()) {
            prop_assert_eq!(x.user_id, y.user_id);
            prop_assert_eq!(x.net_cents, y.net_cents);
        }
        prop_assert_eq!(a.transfers.len(), b.transfers.len());
        for (x, y) in a.transfers.iter().zip(b.transfers.iter()) {
            prop_assert_eq!(x.from_user_id, y.from_user_id);
            prop_assert_eq!(x.to_user_id, y.to_user_id);
            prop_assert_eq!(x.amount_cents, y.amount_cents);
            prop_assert_eq!(x.seq, y.seq);
        }
    }
}
