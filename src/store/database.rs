//! Database-backed persistence for the event store.

use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};

use super::{Store, StoreData, StoreEvent};
use crate::{AppError, ElectionConfig};

#[cfg(feature = "database")]
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for StoreEvent<Vec<u8>> {
    /// Map a database row into a store event with an encrypted payload.
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use chrono::{DateTime, Utc};
        use sqlx::Row;

        let event_id: i64 = row.try_get("event_id")?;
        let payload: Vec<u8> = row.try_get("payload")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;

        Ok(Self {
            event_id: event_id as usize,
            payload,
            created_at,
        })
    }
}

/// Initialize the database schema for event and session persistence.
#[cfg(feature = "migrations")]
pub async fn migrate(pool: &sqlx::PgPool) -> Result<(), AppError> {
    const MIGRATION_LOCK_KEY: i64 = 0x454B53544F52454E; // "EKSTOREN" advisory lock key

    let mut conn = pool.acquire().await?;
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(MIGRATION_LOCK_KEY)
        .execute(&mut *conn)
        .await?;

    let result = async {
        create_streams_table(&mut conn).await?;
        create_events_table(&mut conn).await?;
        create_sessions_table(&mut conn).await?;
        Ok::<(), AppError>(())
    }
    .await;

    let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(MIGRATION_LOCK_KEY)
        .execute(&mut *conn)
        .await;

    result
}

#[cfg(feature = "migrations")]
async fn create_streams_table(conn: &mut sqlx::PgConnection) -> Result<(), AppError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS streams (
          stream_id UUID NOT NULL,
          election TEXT NOT NULL,
          last_event_id BIGINT NOT NULL,
          PRIMARY KEY (stream_id, election)
        )
        "#,
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

#[cfg(feature = "migrations")]
async fn create_events_table(conn: &mut sqlx::PgConnection) -> Result<(), AppError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS events (
          stream_id UUID NOT NULL,
          election TEXT NOT NULL,
          event_id BIGINT NOT NULL,
          created_at timestamp with time zone NOT NULL,
          payload bytea NOT NULL,
          PRIMARY KEY (stream_id, election, event_id)
        )
        "#,
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

#[cfg(feature = "migrations")]
async fn create_sessions_table(conn: &mut sqlx::PgConnection) -> Result<(), AppError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
          token TEXT PRIMARY KEY,
          stream_id UUID,
          current_election JSONB,
          locale TEXT NOT NULL,
          csrf_token TEXT NOT NULL,
          last_activity TIMESTAMPTZ NOT NULL
        )
        "#,
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS sessions_last_activity_idx
           ON sessions(last_activity)"#,
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Ensure a stream row exists for the given (stream_id, election).
pub async fn ensure_stream(
    pool: &sqlx::PgPool,
    stream_id: uuid::Uuid,
    election: ElectionConfig,
) -> Result<(), AppError> {
    sqlx::query(
        r#"INSERT INTO streams (stream_id, election, last_event_id)
        VALUES ($1, $2, 0)
        ON CONFLICT (stream_id, election) DO NOTHING"#,
    )
    .bind(stream_id)
    .bind(election.stable_id())
    .execute(pool)
    .await?;

    Ok(())
}

/// Check which of the given stream IDs have persisted events in any election.
pub async fn streams_with_data(
    pool: &sqlx::PgPool,
    stream_ids: &[uuid::Uuid],
) -> Result<std::collections::HashSet<uuid::Uuid>, AppError> {
    let rows: Vec<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT DISTINCT stream_id FROM streams
         WHERE stream_id = ANY($1) AND last_event_id > 0",
    )
    .bind(stream_ids)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
}

/// List the elections that have persisted events under the given stream.
pub async fn elections_for_stream(
    pool: &sqlx::PgPool,
    stream_id: uuid::Uuid,
) -> Result<Vec<ElectionConfig>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT election FROM streams
         WHERE stream_id = $1 AND last_event_id > 0",
    )
    .bind(stream_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(code,)| parse_stable_id(&code))
        .collect())
}

/// Parse a `stable_id()` string (e.g. `"EK27"`, `"PS27:GR"`) back to an `ElectionConfig`.
fn parse_stable_id(value: &str) -> Option<ElectionConfig> {
    let (code, region) = match value.split_once(':') {
        Some((code, region)) => (code, Some(region)),
        None => (value, None),
    };
    ElectionConfig::from_code_and_region(code, region)
}

/// Load and replay missing events from the database into the store.
pub async fn load_from_database<D>(store: &Store<D>, pool: &sqlx::PgPool) -> Result<(), AppError>
where
    D: StoreData,
    D::Event: DeserializeOwned,
{
    let mut tx = pool.begin().await?;

    if let Err(err) = catch_up(store, &mut tx).await {
        tx.rollback().await?;
        return Err(err);
    }

    Ok(())
}

/// Append the event to the database and apply it to the store.
pub async fn update_in_database<D>(
    store: &Store<D>,
    pool: &sqlx::PgPool,
    event: D::Event,
) -> Result<(), AppError>
where
    D: StoreData,
    D::Event: Serialize + DeserializeOwned,
{
    let mut tx = pool.begin().await?;

    let last_id = match catch_up(store, &mut tx).await {
        Ok(id) => id,
        Err(err) => {
            tx.rollback().await?;
            return Err(err);
        }
    };

    let next_id = last_id + 1;

    let store_event = StoreEvent {
        event_id: next_id,
        payload: event,
        created_at: Utc::now(),
    };

    if let Err(err) = append_once(store, next_id, &store_event, &mut tx).await {
        tx.rollback().await?;
        return Err(err);
    }

    tx.commit().await?;

    store.apply_event(next_id, store_event);

    Ok(())
}

/// Bring the in-memory store up to date by replaying missing events.
async fn catch_up<D>(
    store: &Store<D>,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<usize, AppError>
where
    D: StoreData,
    D::Event: DeserializeOwned,
{
    let last_id: usize = store.data.read().last_event_id();
    let election_id = store.election.stable_id();

    let stream_last_id: i64 = sqlx::query_scalar(
        r#"SELECT last_event_id
        FROM streams
        WHERE stream_id = $1 AND election = $2
        FOR UPDATE"#,
    )
    .bind(store.stream_id)
    .bind(&election_id)
    .fetch_one(&mut **tx)
    .await?;

    let missing: Vec<StoreEvent<Vec<u8>>> = sqlx::query_as::<_, StoreEvent<Vec<u8>>>(
        r#"
        SELECT event_id, payload, created_at
        FROM events
        WHERE stream_id = $1 AND election = $2 AND event_id > $3
        ORDER BY event_id ASC
        "#,
    )
    .bind(store.stream_id)
    .bind(&election_id)
    .bind(last_id as i64)
    .fetch_all(&mut **tx)
    .await?;

    let mut data = store.data.write();

    for event in missing {
        if data.last_event_id() >= event.event_id {
            continue;
        }

        let payload = store.cipher.decrypt_owned::<D::Event>(event.payload)?;
        data.apply(StoreEvent {
            event_id: event.event_id,
            payload,
            created_at: event.created_at,
        });
    }

    Ok(stream_last_id as usize)
}

/// Append a single event to the database within an open transaction.
async fn append_once<D, E: Serialize>(
    store: &Store<D>,
    next_id: usize,
    event: &StoreEvent<E>,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), AppError>
where
    D: StoreData,
{
    let encrypted_payload = store.cipher.encrypt(&event.payload)?;
    let election_id = store.election.stable_id();

    sqlx::query(
        r#"INSERT INTO events (stream_id, election, event_id, created_at, payload)
        VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(store.stream_id)
    .bind(&election_id)
    .bind(next_id as i64)
    .bind(event.created_at)
    .bind(encrypted_payload)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"UPDATE streams SET last_event_id = $3
           WHERE stream_id = $1 AND election = $2"#,
    )
    .bind(store.stream_id)
    .bind(&election_id)
    .bind(next_id as i64)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
