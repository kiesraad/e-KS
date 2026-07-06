//! Database-backed persistence for the event store.

use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};

use super::{
    Store, StoreData, StoreEvent, StreamMeta, chain_hash, encryption::EventCipher, event_aad,
};
use crate::{AppError, ElectionConfig, Scope, StreamId};

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
        create_pending_requests_table(&mut conn).await?;
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
          scope TEXT NOT NULL DEFAULT 'political_group',
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
    // `token` holds the token's SHA-256 hash, not the token itself.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
          token TEXT PRIMARY KEY,
          stream_id UUID,
          current_election JSONB,
          locale TEXT NOT NULL,
          csrf_token TEXT NOT NULL,
          last_activity TIMESTAMPTZ NOT NULL,
          saml_name_id TEXT,
          scope TEXT NOT NULL DEFAULT 'political_group',
          created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
          user_agent_hash TEXT
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

#[cfg(feature = "migrations")]
async fn create_pending_requests_table(conn: &mut sqlx::PgConnection) -> Result<(), AppError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pending_requests (
          id TEXT PRIMARY KEY,
          created_at TIMESTAMPTZ NOT NULL
        )
        "#,
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS pending_requests_created_at_idx
           ON pending_requests(created_at)"#,
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Probe every table the application depends on, so a missing or broken schema
/// (for example a dropped `sessions` table) surfaces as an error rather than a
/// confusing per-request failure later
pub async fn verify_schema(pool: &sqlx::PgPool) -> Result<(), AppError> {
    // `LIMIT 0` checks each table exists and is readable without scanning rows.
    const TABLE_PROBES: [&str; 4] = [
        "SELECT 1 FROM streams LIMIT 0",
        "SELECT 1 FROM events LIMIT 0",
        "SELECT 1 FROM sessions LIMIT 0",
        "SELECT 1 FROM pending_requests LIMIT 0",
    ];

    for probe in TABLE_PROBES {
        sqlx::query(probe).execute(pool).await?;
    }
    Ok(())
}

/// Ensure a stream row exists for the given (stream_id, election), recording its
/// `scope`. The scope is fixed when the row is first created (a stream is only
/// ever used by one store type); later calls leave the existing scope untouched.
pub async fn ensure_stream(
    pool: &sqlx::PgPool,
    stream_id: StreamId,
    election: ElectionConfig,
    scope: Scope,
) -> Result<(), AppError> {
    sqlx::query(
        r#"INSERT INTO streams (stream_id, election, last_event_id, scope)
        VALUES ($1, $2, 0, $3)
        ON CONFLICT (stream_id, election) DO NOTHING"#,
    )
    .bind(stream_id.uuid())
    .bind(election.stable_id())
    .bind(scope.as_str())
    .execute(pool)
    .await?;

    Ok(())
}

/// List every `(stream_id, election)` stream with the given scope that has
/// persisted data.
///
/// A stream is identified by its `(stream_id, election)` pair, so a single
/// `stream_id` can appear multiple times, once per election. Empty placeholder
/// rows (`last_event_id = 0`) are excluded, mirroring [`elections_for_stream`].
pub async fn streams_by_scope(
    pool: &sqlx::PgPool,
    scope: Scope,
) -> Result<Vec<(StreamId, ElectionConfig)>, AppError> {
    let rows: Vec<(uuid::Uuid, String)> = sqlx::query_as(
        r#"SELECT stream_id, election
           FROM streams
           WHERE scope = $1 AND last_event_id > 0"#,
    )
    .bind(scope.as_str())
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(id, code)| parse_stable_id(&code).map(|election| (StreamId(id), election)))
        .collect())
}

/// List [`StreamMeta`] for every stream with the given scope in one aggregate
/// query. `event_count` is `streams.last_event_id`; the timestamps are
/// `MIN`/`MAX` over the events' plain `created_at`. Empty placeholders
/// (`last_event_id = 0`) are excluded, mirroring [`streams_by_scope`].
pub async fn stream_metadata_by_scope(
    pool: &sqlx::PgPool,
    scope: Scope,
) -> Result<Vec<StreamMeta>, AppError> {
    /// `(stream_id, election, last_event_id, first created_at, last created_at)`.
    type MetaRow = (
        uuid::Uuid,
        String,
        i64,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
    );

    let rows: Vec<MetaRow> = sqlx::query_as(
        r#"SELECT s.stream_id, s.election, s.last_event_id,
                      MIN(e.created_at) AS created_at, MAX(e.created_at) AS last_event_at
               FROM streams s
               LEFT JOIN events e
                 ON e.stream_id = s.stream_id AND e.election = s.election
               WHERE s.scope = $1 AND s.last_event_id > 0
               GROUP BY s.stream_id, s.election, s.last_event_id"#,
    )
    .bind(scope.as_str())
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(id, code, last_id, created_at, last_event_at)| {
            parse_stable_id(&code).map(|election| StreamMeta {
                stream_id: StreamId(id),
                election,
                event_count: last_id as usize,
                created_at,
                last_event_at,
            })
        })
        .collect())
}

/// List the elections that have persisted events under the given stream.
pub async fn elections_for_stream(
    pool: &sqlx::PgPool,
    stream_id: StreamId,
) -> Result<Vec<ElectionConfig>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT election FROM streams
         WHERE stream_id = $1 AND last_event_id > 0",
    )
    .bind(stream_id.uuid())
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(code,)| parse_stable_id(&code))
        .collect())
}

/// Locate the political-group event whose chain hash begins with `hash_prefix`.
///
/// Returns the `(stream_id, election, event_id)` of the single matching event,
/// or `None` if nothing matches. The lookup is restricted to
/// [`Scope::PoliticalGroup`] streams so a prefix can only ever resolve to an
/// app-store event (never a CSB event). The prefix match uses a `substring`
/// comparison, which cannot use the `events_hash_idx` btree, so it scans the
/// `events` table; acceptable at current volumes, but revisit with a
/// left-anchored range predicate if it grows. An ambiguous prefix matching
/// more than one event is reported as [`AppError::AmbiguousHash`].
pub async fn find_event_by_hash_prefix(
    pool: &sqlx::PgPool,
    hash_prefix: &[u8],
) -> Result<Option<(StreamId, ElectionConfig, usize)>, AppError> {
    let rows: Vec<(uuid::Uuid, String, i64)> = sqlx::query_as(
        r#"
        SELECT e.stream_id, e.election, e.event_id
        FROM events e
        JOIN streams s ON s.stream_id = e.stream_id AND s.election = e.election
        WHERE s.scope = $1
          AND substring(e.hash from 1 for octet_length($2)) = $2
        LIMIT 2
        "#,
    )
    .bind(Scope::PoliticalGroup.as_str())
    .bind(hash_prefix)
    .fetch_all(pool)
    .await?;

    if rows.len() > 1 {
        return Err(AppError::AmbiguousHash);
    }

    Ok(rows
        .into_iter()
        .next()
        .and_then(|(stream_id, code, event_id)| {
            parse_stable_id(&code)
                .map(|election| (StreamId(stream_id), election, event_id as usize))
        }))
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
pub async fn load_from_database<D>(
    store: &Store<D>,
    pool: &sqlx::PgPool,
    cipher: &EventCipher,
) -> Result<(), AppError>
where
    D: StoreData,
    D::Event: DeserializeOwned,
{
    let mut tx = pool.begin().await?;

    if let Err(err) = catch_up(store, &mut tx, cipher).await {
        tx.rollback().await?;
        return Err(err);
    }

    Ok(())
}

/// Append the event to the database and apply it to the store.
pub async fn update_in_database<D>(
    store: &Store<D>,
    pool: &sqlx::PgPool,
    cipher: &EventCipher,
    event: D::Event,
) -> Result<(), AppError>
where
    D: StoreData,
    D::Event: Serialize + DeserializeOwned,
{
    let mut tx = pool.begin().await?;

    let last_id = match catch_up(store, &mut tx, cipher).await {
        Ok(id) => id,
        Err(err) => {
            tx.rollback().await?;
            return Err(err);
        }
    };

    let next_id = last_id + 1;
    let created_at = Utc::now();
    let prev_hash = store.data.read().last_event_hash();

    let hash = match insert_event(
        store, cipher, next_id, created_at, &event, &prev_hash, &mut tx,
    )
    .await
    {
        Ok(hash) => hash,
        Err(err) => {
            tx.rollback().await?;
            return Err(err);
        }
    };

    tx.commit().await?;

    store.apply_persisted_event(next_id, event, created_at, hash);

    Ok(())
}

/// Bring the in-memory store up to date by replaying missing events.
async fn catch_up<D>(
    store: &Store<D>,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    cipher: &EventCipher,
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
    .bind(store.stream_id.uuid())
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
    .bind(store.stream_id.uuid())
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
        let payload = cipher.decrypt_owned::<D::Event>(event.payload, &aad)?;
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
    cipher: &EventCipher,
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
    let encrypted_payload = cipher.encrypt(payload, &aad)?;
    let hash = chain_hash(prev_hash, event_id, created_at, &encrypted_payload);
    let election_id = store.election.stable_id();

    sqlx::query(
        r#"INSERT INTO events (stream_id, election, event_id, created_at, hash, payload)
        VALUES ($1, $2, $3, $4, $5, $6)"#,
    )
    .bind(store.stream_id.uuid())
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
    .bind(store.stream_id.uuid())
    .bind(&election_id)
    .bind(event_id as i64)
    .execute(&mut **tx)
    .await?;

    Ok(hash)
}
