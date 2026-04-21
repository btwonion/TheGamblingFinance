-- =============================================================
-- Phase 0 · Migration 4/5
-- Settlement materialization.
--
-- Computed once at close, stored so historical reads are cheap
-- and answers remain authoritative even if the algorithm is
-- revised later (algo_version records what produced the row).
--
-- Reopening a night deletes its settlement rows and flips nights.status
-- back to 'open'; the cascade below handles the clean-up.
-- =============================================================

CREATE TABLE settlements (
    night_id        UUID PRIMARY KEY REFERENCES nights(id) ON DELETE CASCADE,
    computed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Bumped whenever backend/src/domain/settlement.rs changes the
    -- output shape or tie-break rule. Frontend may surface it.
    algo_version    SMALLINT NOT NULL
);

-- One row per player per settled night. Positive net = receives,
-- negative = pays. Sum over a night is 0.
CREATE TABLE settlement_balances (
    night_id        UUID NOT NULL REFERENCES nights(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id)  ON DELETE RESTRICT,
    net_cents       BIGINT NOT NULL,
    PRIMARY KEY (night_id, user_id)
);

-- Emitted transfers, in stable `seq` order. Greedy algorithm plus
-- user_id-byte tie-breaking gives deterministic seq.
CREATE TABLE settlement_transfers (
    id              UUID PRIMARY KEY,
    night_id        UUID NOT NULL REFERENCES nights(id) ON DELETE CASCADE,
    from_user_id    UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    to_user_id      UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    amount_cents    BIGINT NOT NULL CHECK (amount_cents > 0),
    seq             INTEGER NOT NULL CHECK (seq >= 0),
    CHECK (from_user_id <> to_user_id),
    UNIQUE (night_id, seq)
);
