-- =============================================================
-- Phase 0 · Migration 3/5
-- Buy-ins, IOU trades, and cash-outs.
--
-- Buy-ins: money from the bank to a player for chips.
-- Trades : IOUs between players; chips move giver -> receiver,
--          receiver owes giver amount_cents_owed at settlement.
-- Cash-outs: one per player per night, exchanges chips back to
--            cash at night.cents_per_chip.
--
-- Composite FKs to night_players ensure every actor is actually
-- in the night. Self-trades and non-positive amounts are rejected
-- at the DB level.
-- =============================================================

CREATE TABLE buy_ins (
    id              UUID PRIMARY KEY,
    night_id        UUID NOT NULL,
    user_id         UUID NOT NULL,
    amount_cents    BIGINT NOT NULL CHECK (amount_cents > 0),
    chips           BIGINT NOT NULL CHECK (chips > 0),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by      UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    -- Membership-enforcing composite FK. Cascade from the night.
    FOREIGN KEY (night_id, user_id)
        REFERENCES night_players(night_id, user_id)
        ON DELETE CASCADE
);

CREATE TABLE trades (
    id                  UUID PRIMARY KEY,
    night_id            UUID NOT NULL,
    chip_giver_id       UUID NOT NULL,
    chip_receiver_id    UUID NOT NULL,
    chips               BIGINT NOT NULL CHECK (chips > 0),
    amount_cents_owed   BIGINT NOT NULL CHECK (amount_cents_owed > 0),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by          UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    CHECK (chip_giver_id <> chip_receiver_id),
    -- Both sides must be members of the night.
    FOREIGN KEY (night_id, chip_giver_id)
        REFERENCES night_players(night_id, user_id)
        ON DELETE CASCADE,
    FOREIGN KEY (night_id, chip_receiver_id)
        REFERENCES night_players(night_id, user_id)
        ON DELETE CASCADE
);

CREATE TABLE cash_outs (
    id              UUID PRIMARY KEY,
    night_id        UUID NOT NULL,
    user_id         UUID NOT NULL,
    -- chips can be zero: "I lost it all, cashing out with nothing".
    chips           BIGINT NOT NULL CHECK (chips >= 0),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by      UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    -- Exactly one cash-out per player per night. PUT semantics on
    -- the upsert endpoint rely on this.
    UNIQUE (night_id, user_id),
    FOREIGN KEY (night_id, user_id)
        REFERENCES night_players(night_id, user_id)
        ON DELETE CASCADE
);
