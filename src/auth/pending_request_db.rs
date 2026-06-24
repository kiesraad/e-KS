//! Postgres-backed persistence for outstanding AuthnRequest IDs.
//!
//! Keeps all `sqlx` usage in one file so the `pending_request_store` module
//! depends only on the generic types.

#![cfg(feature = "database")]

use auth_service::PENDING_REQUEST_TTL;
use chrono::{DateTime, Duration, Utc};

use crate::AppError;

/// Timestamp before which a pending request is considered expired (eID §7.5:
/// artifacts are valid for at most 15 minutes, matching the in-memory store).
fn cutoff() -> DateTime<Utc> {
    Utc::now() - Duration::seconds(PENDING_REQUEST_TTL.as_secs() as i64)
}

/// Record an outgoing AuthnRequest ID, sweeping entries past the TTL first so
/// abandoned flows cannot accumulate (mirrors the in-memory store).
pub async fn register(pool: &sqlx::PgPool, id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM pending_requests WHERE created_at < $1")
        .bind(cutoff())
        .execute(pool)
        .await?;

    sqlx::query(
        r#"INSERT INTO pending_requests (id, created_at) VALUES ($1, $2)
           ON CONFLICT (id) DO UPDATE SET created_at = EXCLUDED.created_at"#,
    )
    .bind(id)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(())
}

/// Atomically validate and consume a matched AuthnRequest ID (eID §7.6.3.5
/// rule 4 / §9.7): delete the row only if it exists and is still within the TTL
/// window, reporting whether a row matched. A single statement, so the check and
/// the consume cannot race, two concurrent ACS callbacks for the same artifact
/// can never both succeed.
pub async fn consume_if_pending(pool: &sqlx::PgPool, id: &str) -> Result<bool, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"DELETE FROM pending_requests
           WHERE id = $1 AND created_at >= $2
           RETURNING id"#,
    )
    .bind(id)
    .bind(cutoff())
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}
