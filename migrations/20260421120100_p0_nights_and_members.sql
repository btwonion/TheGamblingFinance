-- =============================================================
-- Phase 0 · Migration 2/5
-- Poker nights and their player membership.
--
-- "Nights" (not "sessions") avoids collision with auth sessions.
-- cents_per_chip is fixed per night, set at night creation.
-- Currency column is reserved for future multi-currency support;
-- all Phase 0 nights are EUR.
-- =============================================================

CREATE TABLE nights (
    id              UUID PRIMARY KEY,
    created_by      UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    title           TEXT NOT NULL CHECK (char_length(title) BETWEEN 1 AND 120),
    played_on       DATE NOT NULL,
    currency        CHAR(3) NOT NULL DEFAULT 'EUR'
                        CHECK (currency ~ '^[A-Z]{3}$'),
    cents_per_chip  INTEGER NOT NULL CHECK (cents_per_chip > 0),
    status          TEXT NOT NULL DEFAULT 'open'
                        CHECK (status IN ('open', 'closed')),
    opened_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at       TIMESTAMPTZ NULL,
    notes           TEXT NULL,
    CHECK ((status = 'open'  AND closed_at IS NULL)
        OR (status = 'closed' AND closed_at IS NOT NULL))
);

-- Membership. Composite PK so (night_id, user_id) is unique, and
-- the activity tables can reference it with a composite FK to
-- enforce "player must be a member of the night".
CREATE TABLE night_players (
    night_id        UUID NOT NULL REFERENCES nights(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id)  ON DELETE RESTRICT,
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (night_id, user_id)
);
