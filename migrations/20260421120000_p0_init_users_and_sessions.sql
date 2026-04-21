-- =============================================================
-- Phase 0 · Migration 1/5
-- Users and auth sessions.
--
-- UUID v7 IDs are generated client-side in Rust; no DB default.
-- Passwords stored as argon2id PHC strings in `password_hash`.
-- auth_sessions stores SHA-256 of the cookie value — the raw
-- token is never persisted.
-- =============================================================

-- citext gives us case-insensitive email uniqueness without a
-- functional index and without normalizing in application code.
CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE users (
    id              UUID PRIMARY KEY,
    email           CITEXT NOT NULL UNIQUE,
    display_name    TEXT NOT NULL CHECK (char_length(display_name) BETWEEN 1 AND 80),
    password_hash   TEXT NOT NULL,
    role            TEXT NOT NULL CHECK (role IN ('admin', 'player')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Soft-disable: admins flip this column; deletion is blocked by
    -- FK RESTRICT on financial tables.
    disabled_at     TIMESTAMPTZ NULL
);

CREATE TABLE auth_sessions (
    id              UUID PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    -- SHA-256 hex of the raw cookie value. 64 chars.
    token_hash      TEXT NOT NULL UNIQUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,
    last_seen_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    user_agent      TEXT NULL,
    ip_addr         INET NULL,
    revoked_at      TIMESTAMPTZ NULL
);
