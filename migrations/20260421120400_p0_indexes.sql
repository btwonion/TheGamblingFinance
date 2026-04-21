-- =============================================================
-- Phase 0 · Migration 5/5
-- Indexes supporting the hot read paths.
--
-- Ownership lookups (email at login, member nights) and
-- leaderboard aggregation drive these choices.
-- =============================================================

-- Auth: fast expiry sweeps and "is my cookie still valid" checks.
CREATE INDEX idx_auth_sessions_expires_at  ON auth_sessions(expires_at);
CREATE INDEX idx_auth_sessions_user_id     ON auth_sessions(user_id);

-- Nights: dashboard lists open nights; history view orders by date.
CREATE INDEX idx_nights_status     ON nights(status);
CREATE INDEX idx_nights_played_on  ON nights(played_on DESC);

-- Night membership: "which nights am I in?" for the player view.
CREATE INDEX idx_night_players_user_id ON night_players(user_id);

-- Activity tables: fan-out from a night_id is the dominant access.
CREATE INDEX idx_buy_ins_night_id   ON buy_ins(night_id);
CREATE INDEX idx_trades_night_id    ON trades(night_id);
CREATE INDEX idx_cash_outs_night_id ON cash_outs(night_id);

-- Settlement: lifetime leaderboard sums net_cents per user across
-- all closed nights, so user_id is the hot column.
CREATE INDEX idx_settlement_balances_user_id ON settlement_balances(user_id);

-- Transfers: "my pending/past debts" may want per-user filters.
CREATE INDEX idx_settlement_transfers_night_id ON settlement_transfers(night_id);
CREATE INDEX idx_settlement_transfers_from     ON settlement_transfers(from_user_id);
CREATE INDEX idx_settlement_transfers_to       ON settlement_transfers(to_user_id);
