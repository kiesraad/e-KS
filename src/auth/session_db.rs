//! Postgres-backed session persistence helpers.
//!
//! Keeps all `sqlx` / `serde_json` usage in one file so the `session_store`
//! module itself depends only on the generic session types.

#![cfg(feature = "database")]

use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::{
    AppError, ElectionConfig, Locale, Scope, Session, StreamId, TokenValue,
    auth::session::{session_absolute_timeout, session_idle_timeout},
};

/// A `sessions` row, mapped by column name. `token` holds the token hash.
#[derive(sqlx::FromRow)]
struct SessionRow {
    token: String,
    stream_id: Option<uuid::Uuid>,
    paper_correction_stream_id: Option<uuid::Uuid>,
    current_election: Option<serde_json::Value>,
    locale: String,
    last_activity: DateTime<Utc>,
    saml_name_id: String,
    scope: String,
    created_at: DateTime<Utc>,
    user_agent_hash: Option<String>,
    csrf_token: String,
}

/// Insert or update a session row (`token` column holds the hash). `created_at`
/// and `user_agent_hash` are omitted from `ON CONFLICT` so a touch can't reset
/// them.
pub async fn upsert(pool: &sqlx::PgPool, session: &Session) -> Result<(), AppError> {
    let current_election_json = session
        .current_election
        .map(serde_json::to_value)
        .transpose()?;

    sqlx::query(
        r#"
        INSERT INTO sessions
            (token, stream_id, paper_correction_stream_id, current_election, locale, last_activity, saml_name_id, scope, created_at, user_agent_hash, csrf_token)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (token) DO UPDATE SET
            stream_id = EXCLUDED.stream_id,
            paper_correction_stream_id = EXCLUDED.paper_correction_stream_id,
            current_election = EXCLUDED.current_election,
            locale = EXCLUDED.locale,
            last_activity = EXCLUDED.last_activity,
            saml_name_id = EXCLUDED.saml_name_id,
            scope = EXCLUDED.scope,
            csrf_token = EXCLUDED.csrf_token
        "#,
    )
    .bind(session.token_hash())
    .bind(session.stream_id.map(|s| s.uuid()))
    .bind(session.paper_correction_stream_id.map(|s| s.uuid()))
    .bind(current_election_json)
    .bind(session.locale.as_str())
    .bind(session.last_activity)
    .bind(&session.saml_name_id)
    .bind(session.scope.as_str())
    .bind(session.created_at)
    .bind(&session.user_agent_hash)
    .bind(&session.csrf_token().0)
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetch a single session by its token hash.
pub async fn load(pool: &sqlx::PgPool, token_hash: &str) -> Result<Option<Session>, AppError> {
    let row: Option<SessionRow> = sqlx::query_as(
        r#"SELECT token, stream_id, paper_correction_stream_id, current_election, locale, last_activity, saml_name_id, scope, created_at, user_agent_hash, csrf_token
           FROM sessions WHERE token = $1"#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;

    row.map(session_from_row).transpose()
}

fn session_from_row(row: SessionRow) -> Result<Session, AppError> {
    let current_election = row
        .current_election
        .map(serde_json::from_value::<ElectionConfig>)
        .transpose()?;

    Ok(Session {
        token_hash: row.token,
        raw_token: None, // never carried by a reloaded session
        csrf_token: TokenValue(row.csrf_token),
        created_at: row.created_at,
        last_activity: row.last_activity,
        user_agent_hash: row.user_agent_hash,
        stream_id: row.stream_id.map(StreamId::from),
        paper_correction_stream_id: row.paper_correction_stream_id.map(StreamId::from),
        scope: Scope::from_str(&row.scope).unwrap_or_default(),
        current_election,
        locale: Locale::from_str(&row.locale).unwrap_or_default(),
        saml_name_id: row.saml_name_id,
    })
}

/// Write the mutable fields of an existing session row. Like [`upsert`] but a
/// plain UPDATE, so it never re-creates a row a concurrent logout deleted.
/// `created_at`, `user_agent_hash` and `saml_name_id` are fixed at login.
pub async fn update(pool: &sqlx::PgPool, session: &Session) -> Result<(), AppError> {
    let current_election_json = session
        .current_election
        .map(serde_json::to_value)
        .transpose()?;

    sqlx::query(
        r#"
        UPDATE sessions SET
            stream_id = $2,
            paper_correction_stream_id = $3,
            current_election = $4,
            locale = $5,
            last_activity = $6,
            scope = $7,
            csrf_token = $8
        WHERE token = $1
        "#,
    )
    .bind(session.token_hash())
    .bind(session.stream_id.map(|s| s.uuid()))
    .bind(session.paper_correction_stream_id.map(|s| s.uuid()))
    .bind(current_election_json)
    .bind(session.locale.as_str())
    .bind(session.last_activity)
    .bind(session.scope.as_str())
    .bind(&session.csrf_token().0)
    .execute(pool)
    .await?;

    Ok(())
}

/// Refresh `last_activity` of an existing session row. A conditional UPDATE
/// (not an upsert), so it never re-creates a row a concurrent logout deleted.
pub async fn touch(
    pool: &sqlx::PgPool,
    token_hash: &str,
    last_activity: DateTime<Utc>,
) -> Result<(), AppError> {
    sqlx::query("UPDATE sessions SET last_activity = $2 WHERE token = $1")
        .bind(token_hash)
        .bind(last_activity)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete a single session by its token hash.
pub async fn delete(pool: &sqlx::PgPool, token_hash: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE token = $1")
        .bind(token_hash)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete all sessions that have passed either the idle timeout or the absolute
/// lifetime cap (mirrors [`Session::is_expired`]).
pub async fn cleanup_expired(pool: &sqlx::PgPool) -> Result<(), AppError> {
    let now = Utc::now();
    let idle_cutoff = now - session_idle_timeout();
    let absolute_cutoff = now - session_absolute_timeout();
    sqlx::query("DELETE FROM sessions WHERE last_activity < $1 OR created_at < $2")
        .bind(idle_cutoff)
        .bind(absolute_cutoff)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_row(saml_name_id: String) -> SessionRow {
        SessionRow {
            token: "token-hash-abc".to_string(),
            stream_id: None,
            paper_correction_stream_id: None,
            current_election: None,
            locale: Locale::default().as_str().to_string(),
            last_activity: Utc::now(),
            saml_name_id,
            scope: Scope::default().as_str().to_string(),
            created_at: Utc::now(),
            user_agent_hash: Some("ua-hash".to_string()),
            csrf_token: "csrf-token-abc".to_string(),
        }
    }

    /// The SAML NameID survives the row to `Session` mapping so SP-initiated
    /// logout (eID §7.7.1) still works for database-backed sessions.
    #[test]
    fn session_from_row_preserves_saml_name_id() {
        let session = session_from_row(sample_row("name-id-xyz".to_string())).expect("maps row");
        assert_eq!(session.saml_name_id, "name-id-xyz");
    }

    /// An empty NameID (dev-login/pre-auth) maps through unchanged.
    #[test]
    fn session_from_row_maps_empty_saml_name_id() {
        let session = session_from_row(sample_row(String::new())).expect("maps row");
        assert!(session.saml_name_id.is_empty());
    }
}
