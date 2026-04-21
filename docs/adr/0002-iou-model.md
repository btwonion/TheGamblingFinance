# ADR 0002 — IOU trade model and settlement algorithm

- **Status:** accepted
- **Date:** 2026-04-21
- **Deciders:** anton

## Context

At a home poker night, chips pass between players mid-hand for cash debts
("Alice trades Bob 5 € for 10 of Bob's chips"). Two modeling choices:

1. **Cash-move model** — every trade is an immediate cash settlement
   between the two players. Simpler arithmetic, but requires players to
   carry cash and track it physically during the game.
2. **IOU model** — chips move immediately, but the cash debt is recorded
   and settled with all other debts **once at night-end**. Matches how
   people actually play: one settlement conversation after the last hand.

## Decision

We adopt the **IOU model**. No cash moves until the night closes; the app
is authoritative on who owes whom. At close, we solve a min-cashflow
problem across all net positions (buy-ins, cash-outs, IOUs) and emit a
small set of transfers.

## Data model consequences

- `trades` table stores `chip_giver_id`, `chip_receiver_id`, `chips`,
  `amount_cents_owed`. Semantics: chips move giver → receiver; receiver
  owes giver `amount_cents_owed` at settlement.
- A `CHECK (chip_giver_id <> chip_receiver_id)` enforces non-self-trades
  at the DB layer.
- Trades are **never** reflected as cash entries on the bank side; they
  live only on the player↔player ledger.

## Settlement algorithm (normative)

The backend function is:

```rust
pub fn settle(input: &SettlementInput) -> Result<SettlementOutput, SettleError>;
```

**Step 1 — per-player net cents:**

```
cash_value(p)   = cash_outs[p].chips * night.cents_per_chip
buy_in_total(p) = sum of p's buy_ins.amount_cents
iou_received(p) = sum of trade.cents_owed WHERE trade.chip_giver    == p
iou_owed(p)     = sum of trade.cents_owed WHERE trade.chip_receiver == p
net(p) = cash_value(p) - buy_in_total(p) + iou_received(p) - iou_owed(p)
```

Invariant before emitting transfers: **`sum(net) == 0`**. If not, return
`SettleError::UnbalancedLedger { diff_cents }`. The UI blocks close and
renders an actionable error (e.g. "Books off by 2,30 € — check
cash-outs").

**Step 2 — greedy min-cashflow:**

```
creditors = players with net > 0  (largest first)
debtors   = players with net < 0  (largest magnitude first)
while both non-empty:
    pay = min(-debtors[0].net, creditors[0].net)
    transfers.push(Transfer {
        from: debtors[0],
        to: creditors[0],
        amount_cents: pay,
    })
    adjust both, drop any whose net is now 0
```

Produces ≤ N−1 transfers. Not provably optimal in the general case
(min-cashflow is NP-hard), but optimal for home-poker shapes and
explainable to humans.

**Determinism.** Tie-breaks on equal magnitudes use `user_id` byte order
so the stored `seq` column is stable across re-computations. The frontend
may rely on this order for display.

## Edge cases

| Case | Behavior |
|---|---|
| Single-player night | `transfers = []`; valid |
| Missing cash-out for a player | Close endpoint returns 409 listing missing players |
| Self-trade | Rejected at DB level (`CHECK` constraint) |
| Re-close | `POST /nights/:id/close` is wrapped in a serializable transaction; two racing clicks → one wins, the other gets 409 |
| Reopen | `POST /nights/:id/reopen` deletes `settlements`/`settlement_balances`/`settlement_transfers` rows and flips status back to `open` |

## Frontend contract

**The frontend does not reimplement this algorithm.** It renders the
backend's `SettlementResponse` payload (balances + transfers, in `seq`
order). A frontend parity unit test loads a fixture emitted by the
backend's test suite and asserts the rendered transfer list matches.

## Alternatives considered

- **Cash-move model (option 1).** Rejected: contradicts how our group
  actually plays; would produce dozens of tiny cash movements instead of
  a single end-of-night conversation.
- **Optimal ILP solver for min-cashflow.** Rejected: NP-hard, dependency
  bloat, and the greedy produces the same answer for realistic player
  counts (≤ 10).
