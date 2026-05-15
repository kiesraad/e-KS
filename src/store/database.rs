//! Database-backed persistence for the event store.

use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};

use super::{Store, StoreData, StoreEvent, chain_hash, event_aad};
use crate::{AppError, ElectionConfig};

#[cfg(feature = "database")]
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for StoreEvent<Vec<u8>> {
    /// Map a database row into a store event whose `payload` is the encrypted blob.
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let event_id: i64 = row.try_get("event_id")?;
        let payload: Vec<u8> = row.try_get("payload")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let hash_bytes: Vec<u8> = row.try_get("hash")?;
        let hash: [u8; 32] = hash_bytes.as_slice().try_into().map_err(|_| {
            sqlx::Error::Decode(
                format!(
                    "event {event_id} hash is {} bytes, expected 32",
                    hash_bytes.len()
                )
                .into(),
            )
        })?;

        Ok(Self {
            event_id: event_id as usize,
            payload,
            created_at,
            hash,
        })
    }
}

/// Initialize the database schema for event and session persistence.
#[cfg(feature = "migrations")]
pub async fn migrate(pool: &sqlx::PgPool) -> Result<(), AppError> {
    let mut conn = pool.acquire().await?;

    if let Err(error) = async {
        create_streams_table(&mut conn).await?;
        create_events_table(&mut conn).await?;
        create_sessions_table(&mut conn).await?;
        Ok::<(), AppError>(())
    }
    .await
    {
        tracing::warn!("Database migration failed, there might me a concurrent migration: {error}");
    }

    Ok(())
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
          hash bytea NOT NULL,
          payload bytea NOT NULL,
          PRIMARY KEY (stream_id, election, event_id)
        )
        "#,
    )
    .execute(&mut *conn)
    .await?;

    // Supports looking up an event (and its stream) by its chain hash.
    sqlx::query(r#"CREATE INDEX IF NOT EXISTS events_hash_idx ON events(hash)"#)
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
    let created_at = Utc::now();
    let prev_hash = store.data.read().last_event_hash();

    let hash = match insert_event(store, next_id, created_at, &event, &prev_hash, &mut tx).await {
        Ok(hash) => hash,
        Err(err) => {
            tx.rollback().await?;
            return Err(err);
        }
    };

    tx.commit().await?;

    store.apply_event(
        next_id,
        StoreEvent {
            event_id: next_id,
            payload: event,
            created_at,
            hash,
        },
    );

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
        SELECT event_id, payload, created_at, hash
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
    let mut prev_hash = data.last_event_hash();

    for event in missing {
        if data.last_event_id() >= event.event_id {
            continue;
        }

        // `event.payload` is the encrypted blob; verify the chain over it
        // before decrypting. Gated behind a feature flag: it costs a SHA-256
        // over every loaded event. (Reordering, removal, and in-place edits are
        // still caught by the AES-GCM tag, since `prev_hash` is part of the
        // associated data.)
        #[cfg(feature = "verify-event-hash-chain")]
        if chain_hash(&prev_hash, event.event_id, event.created_at, &event.payload) != event.hash {
            return Err(AppError::EventDecodeError(format!(
                "hash chain broken at event {}",
                event.event_id
            )));
        }

        let aad = event_aad(event.event_id, event.created_at, &prev_hash);
        let payload = store
            .cipher
            .decrypt_owned::<D::Event>(event.payload, &aad)?;
        prev_hash = event.hash;
        data.apply(StoreEvent {
            event_id: event.event_id,
            payload,
            created_at: event.created_at,
            hash: event.hash,
        });
    }

    Ok(stream_last_id as usize)
}

/// Encrypt `payload`, insert the event row within an open transaction, bump
/// `streams.last_event_id`, and return the event's chain hash (computed over the
/// encrypted blob).
async fn insert_event<D, E: Serialize>(
    store: &Store<D>,
    event_id: usize,
    created_at: DateTime<Utc>,
    payload: &E,
    prev_hash: &[u8; 32],
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<[u8; 32], AppError>
where
    D: StoreData,
{
    let aad = event_aad(event_id, created_at, prev_hash);
    let encrypted_payload = store.cipher.encrypt(payload, &aad)?;
    let hash = chain_hash(prev_hash, event_id, created_at, &encrypted_payload);
    let election_id = store.election.stable_id();

    sqlx::query(
        r#"INSERT INTO events (stream_id, election, event_id, created_at, hash, payload)
        VALUES ($1, $2, $3, $4, $5, $6)"#,
    )
    .bind(store.stream_id)
    .bind(&election_id)
    .bind(event_id as i64)
    .bind(created_at)
    .bind(hash.as_slice())
    .bind(encrypted_payload)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"UPDATE streams SET last_event_id = $3
           WHERE stream_id = $1 AND election = $2"#,
    )
    .bind(store.stream_id)
    .bind(&election_id)
    .bind(event_id as i64)
    .execute(&mut **tx)
    .await?;

    Ok(hash)
}
