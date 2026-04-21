//! Postgres-backed session persistence helpers.
//!
//! Keeps all `sqlx` / `serde_json` usage in one file so the `session_store`
//! module itself depends only on the generic session types.

#![cfg(feature = "database")]

use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    AppError, CsrfTokens, ElectionConfig, Locale, Session, StreamId, TokenValue,
    auth::session::{SessionToken, session_idle_timeout},
};

/// Wire-level projection of a CSRF token used when reading/writing the DB.
///
/// Kept separate from the in-memory `CsrfTokens` model so `serde` doesn't
/// have to be implemented on the live `Arc<RwLock<…>>` structure.
#[derive(Serialize, Deserialize)]
struct CsrfTokenRow {
    value: TokenValue,
    expires_at: DateTime<Utc>,
}

type SessionRow = (
    String,
    Option<uuid::Uuid>,
    Option<serde_json::Value>,
    String,
    serde_json::Value,
    DateTime<Utc>,
);

/// Serialize the session's live CSRF tokens to a JSONB-ready value.
fn csrf_tokens_to_json(session: &Session) -> Result<serde_json::Value, AppError> {
    let rows: Vec<CsrfTokenRow> = session
        .csrf_tokens
        .to_map()
        .into_iter()
        .map(|(value, expires_at)| CsrfTokenRow { value, expires_at })
        .collect();

    Ok(serde_json::to_value(&rows)?)
}

/// Insert or update a session row.
pub async fn upsert(pool: &sqlx::PgPool, session: &Session) -> Result<(), AppError> {
    let token = session.token().to_exposed_string();
    let csrf_json = csrf_tokens_to_json(session)?;
    let current_election_json = session
        .current_election
        .map(serde_json::to_value)
        .transpose()?;

    sqlx::query(
        r#"
        INSERT INTO sessions
            (token, stream_id, current_election, locale, csrf_tokens, last_activity)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (token) DO UPDATE SET
            stream_id = EXCLUDED.stream_id,
            current_election = EXCLUDED.current_election,
            locale = EXCLUDED.locale,
            csrf_tokens = EXCLUDED.csrf_tokens,
            last_activity = EXCLUDED.last_activity
        "#,
    )
    .bind(&token)
    .bind(session.stream_id.map(|s| s.uuid()))
    .bind(current_election_json)
    .bind(session.locale.as_str())
    .bind(&csrf_json)
    .bind(session.last_activity)
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetch a single session by token.
pub async fn load(pool: &sqlx::PgPool, token: &str) -> Result<Option<Session>, AppError> {
    let row: Option<SessionRow> = sqlx::query_as(
        r#"SELECT token, stream_id, current_election, locale, csrf_tokens, last_activity
           FROM sessions WHERE token = $1"#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    row.map(session_from_row).transpose()
}

fn session_from_row(row: SessionRow) -> Result<Session, AppError> {
    let (token_str, stream_id_uuid, current_election_json, locale_str, csrf_json, last_activity) =
        row;

    let current_election = current_election_json
        .map(serde_json::from_value::<ElectionConfig>)
        .transpose()?;

    let csrf_rows: Vec<CsrfTokenRow> = serde_json::from_value(csrf_json)?;
    let csrf_map: HashMap<TokenValue, DateTime<Utc>> = csrf_rows
        .into_iter()
        .map(|row| (row.value, row.expires_at))
        .collect();

    Ok(Session {
        token: SessionToken::new(token_str),
        last_activity,
        stream_id: stream_id_uuid.map(StreamId::from),
        current_election,
        locale: Locale::from_str(&locale_str).unwrap_or_default(),
        csrf_tokens: CsrfTokens::from_map(csrf_map),
    })
}

/// Delete a single session by token.
pub async fn delete(pool: &sqlx::PgPool, token: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

/// Targeted update of just the `csrf_tokens` column for a session.
pub async fn sync_csrf(pool: &sqlx::PgPool, session: &Session) -> Result<(), AppError> {
    let token = session.token().to_exposed_string();
    let csrf_json = csrf_tokens_to_json(session)?;

    sqlx::query(r#"UPDATE sessions SET csrf_tokens = $1 WHERE token = $2"#)
        .bind(&csrf_json)
        .bind(&token)
        .execute(pool)
        .await?;

    Ok(())
}

/// Delete all sessions whose `last_activity` has aged past the idle timeout.
pub async fn cleanup_expired(pool: &sqlx::PgPool) -> Result<(), AppError> {
    let cutoff = Utc::now() - session_idle_timeout();
    sqlx::query("DELETE FROM sessions WHERE last_activity < $1")
        .bind(cutoff)
        .execute(pool)
        .await?;
    Ok(())
}
