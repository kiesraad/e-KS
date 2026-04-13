//! Database-backed persistence for the event store.

use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};

use super::{Store, StoreData, StoreEvent};
use crate::AppError;

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

/// Initialize the database schema for event persistence.
#[cfg(feature = "migrations")]
pub async fn migrate(pool: &sqlx::PgPool) -> Result<(), AppError> {
    const MIGRATION_LOCK_KEY: i64 = 0x454B53544F52454E; // "EKSTOREN" advisory lock key

    let mut conn = pool.acquire().await?;
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(MIGRATION_LOCK_KEY)
        .execute(&mut *conn)
        .await?;

    let result = async {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS streams (
              stream_id UUID PRIMARY KEY,
              last_event_id BIGINT NOT NULL
            )
            "#,
        )
        .execute(&mut *conn)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
              stream_id UUID NOT NULL,
              event_id BIGINT NOT NULL,
              created_at timestamp with time zone NOT NULL,
              payload bytea NOT NULL,
              PRIMARY KEY (stream_id, event_id)
            )
            "#,
        )
        .execute(&mut *conn)
        .await?;

        Ok::<(), AppError>(())
    }
    .await;

    let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(MIGRATION_LOCK_KEY)
        .execute(&mut *conn)
        .await;

    result
}

/// Ensure a stream row exists for the given stream ID.
pub async fn ensure_stream(pool: &sqlx::PgPool, stream_id: uuid::Uuid) -> Result<(), AppError> {
    sqlx::query(
        r#"INSERT INTO streams (stream_id, last_event_id)
        VALUES ($1, 0)
        ON CONFLICT (stream_id) DO NOTHING"#,
    )
    .bind(stream_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Check which of the given stream IDs have persisted events.
pub async fn streams_with_data(
    pool: &sqlx::PgPool,
    stream_ids: &[uuid::Uuid],
) -> Result<std::collections::HashSet<uuid::Uuid>, AppError> {
    let rows: Vec<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT stream_id FROM streams WHERE stream_id = ANY($1) AND last_event_id > 0",
    )
    .bind(stream_ids)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
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

    let stream_last_id: i64 = sqlx::query_scalar(
        r#"SELECT last_event_id
        FROM streams
        WHERE stream_id = $1
        FOR UPDATE"#,
    )
    .bind(store.stream_id)
    .fetch_one(&mut **tx)
    .await?;

    let missing: Vec<StoreEvent<Vec<u8>>> = sqlx::query_as::<_, StoreEvent<Vec<u8>>>(
        r#"
        SELECT event_id, payload, created_at
        FROM events
        WHERE stream_id = $1 AND event_id > $2
        ORDER BY event_id ASC
        "#,
    )
    .bind(store.stream_id)
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

    sqlx::query(
        r#"INSERT INTO events (stream_id, event_id, created_at, payload)
        VALUES ($1, $2, $3, $4)"#,
    )
    .bind(store.stream_id)
    .bind(next_id as i64)
    .bind(event.created_at)
    .bind(encrypted_payload)
    .execute(&mut **tx)
    .await?;

    sqlx::query(r#"UPDATE streams SET last_event_id = $2 WHERE stream_id = $1"#)
        .bind(store.stream_id)
        .bind(next_id as i64)
        .execute(&mut **tx)
        .await?;

    Ok(())
}
