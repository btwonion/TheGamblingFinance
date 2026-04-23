//! Queries for leaderboard endpoints (per-night + lifetime).

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::leaderboard::LeaderboardEntry;

/// Per-night ranking, reading the already-materialized
/// `settlement_balances` rows. Order: net_cents DESC, display_name ASC.
pub async fn per_night(
    pool: &PgPool,
    night_id: Uuid,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    sqlx::query_as::<_, LeaderboardEntry>(
        "SELECT u.id AS user_id, u.display_name, sb.net_cents, 1::BIGINT AS nights_played \
         FROM settlement_balances sb \
         JOIN users u ON u.id = sb.user_id \
         WHERE sb.night_id = $1 \
         ORDER BY sb.net_cents DESC, u.display_name ASC",
    )
    .bind(night_id)
    .fetch_all(pool)
    .await
}

/// Lifetime ranking across every closed night. Only `settlement_balances`
/// rows are included (open nights contribute nothing).
pub async fn lifetime(pool: &PgPool) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    sqlx::query_as::<_, LeaderboardEntry>(
        "SELECT u.id AS user_id, u.display_name, \
                COALESCE(SUM(sb.net_cents), 0)::BIGINT AS net_cents, \
                COUNT(DISTINCT sb.night_id)::BIGINT AS nights_played \
         FROM users u \
         LEFT JOIN settlement_balances sb ON sb.user_id = u.id \
         LEFT JOIN nights n ON n.id = sb.night_id AND n.status = 'closed' \
         GROUP BY u.id, u.display_name \
         HAVING COUNT(DISTINCT sb.night_id) > 0 \
         ORDER BY net_cents DESC, u.display_name ASC",
    )
    .fetch_all(pool)
    .await
}
