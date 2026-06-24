//! Postgres-backed session persistence helpers.
//!
//! Keeps all `sqlx` / `serde_json` usage in one file so the `session_store`
//! module itself depends only on the generic session types.

#![cfg(feature = "database")]

use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::{
    AppError, ElectionConfig, Locale, Scope, Session, StreamId, TokenValue,
    auth::session::{SessionToken, session_idle_timeout},
};

type SessionRow = (
    String,
    Option<uuid::Uuid>,
    Option<serde_json::Value>,
    String,
    String,
    DateTime<Utc>,
    String,
);

/// Insert or update a session row.
pub async fn upsert(pool: &sqlx::PgPool, session: &Session) -> Result<(), AppError> {
    let token = session.token().to_exposed_string();
    let current_election_json = session
        .current_election
        .map(serde_json::to_value)
        .transpose()?;

    sqlx::query(
        r#"
        INSERT INTO sessions
            (token, stream_id, current_election, locale, csrf_token, last_activity, scope)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (token) DO UPDATE SET
            stream_id = EXCLUDED.stream_id,
            current_election = EXCLUDED.current_election,
            locale = EXCLUDED.locale,
            last_activity = EXCLUDED.last_activity,
            scope = EXCLUDED.scope
        "#,
    )
    .bind(&token)
    .bind(session.stream_id.map(|s| s.uuid()))
    .bind(current_election_json)
    .bind(session.locale.as_str())
    .bind(&session.csrf_token.0)
    .bind(session.last_activity)
    .bind(session.scope.as_str())
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetch a single session by token.
pub async fn load(pool: &sqlx::PgPool, token: &str) -> Result<Option<Session>, AppError> {
    let row: Option<SessionRow> = sqlx::query_as(
        r#"SELECT token, stream_id, current_election, locale, csrf_token, last_activity, scope
           FROM sessions WHERE token = $1"#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    row.map(session_from_row).transpose()
}

fn session_from_row(row: SessionRow) -> Result<Session, AppError> {
    let (
        token_str,
        stream_id_uuid,
        current_election_json,
        locale_str,
        csrf_token,
        last_activity,
        scope_str,
    ) = row;

    let current_election = current_election_json
        .map(serde_json::from_value::<ElectionConfig>)
        .transpose()?;

    Ok(Session {
        token: SessionToken::new(token_str),
        last_activity,
        stream_id: stream_id_uuid.map(StreamId::from),
        scope: Scope::from_str(&scope_str).unwrap_or_default(),
        current_election,
        locale: Locale::from_str(&locale_str).unwrap_or_default(),
        csrf_token: TokenValue(csrf_token),
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

/// Delete all sessions whose `last_activity` has aged past the idle timeout.
pub async fn cleanup_expired(pool: &sqlx::PgPool) -> Result<(), AppError> {
    let cutoff = Utc::now() - session_idle_timeout();
    sqlx::query("DELETE FROM sessions WHERE last_activity < $1")
        .bind(cutoff)
        .execute(pool)
        .await?;
    Ok(())
}
