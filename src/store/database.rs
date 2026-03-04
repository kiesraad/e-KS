//! Database-backed persistence for the event store.

use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};

use super::{Store, StoreData, StoreEvent};
use crate::AppError;

#[cfg(feature = "database")]
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for StoreEvent<serde_json::Value> {
    /// Map a database row into a store event.
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use chrono::{DateTime, Utc};
        use sqlx::Row;

        let event_id: i64 = row.try_get("event_id")?;
        let payload: serde_json::Value = row.try_get("payload")?;
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
              payload jsonb NOT NULL,
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

    if let Err(err) = append_once(store.stream_id, next_id, &store_event, &mut tx).await {
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

    let missing: Vec<StoreEvent<serde_json::Value>> =
        sqlx::query_as::<_, StoreEvent<serde_json::Value>>(
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
            // This can happen if another instance of the application processed events concurrently
            // and updated the store before this instance could acquire the write lock. In that case,
            // the store is already up-to-date and we can skip applying the event again.
            continue;
        }

        let StoreEvent {
            event_id,
            payload,
            created_at,
        } = event;

        match serde_json::from_value::<D::Event>(payload) {
            Ok(ev) => {
                let store_event = StoreEvent {
                    event_id,
                    payload: ev,
                    created_at,
                };
                data.apply(store_event);
                data.set_last_event_id(event_id);
            }
            Err(e) => {
                tracing::error!("Failed to deserialize event: {e:?}");
                continue;
            }
        }
    }

    Ok(stream_last_id as usize)
}

/// Append a single event to the database within an open transaction.
async fn append_once<E: Serialize>(
    stream_id: uuid::Uuid,
    next_id: usize,
    event: &StoreEvent<E>,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), AppError> {
    let new_payload =
        serde_json::to_value(&event.payload).map_err(|_| AppError::InternalServerError)?;

    sqlx::query(
        r#"INSERT INTO events (stream_id, event_id, created_at, payload)
        VALUES ($1, $2, $3, $4)"#,
    )
    .bind(stream_id)
    .bind(next_id as i64)
    .bind(event.created_at)
    .bind(new_payload)
    .execute(&mut **tx)
    .await?;

    sqlx::query(r#"UPDATE streams SET last_event_id = $2 WHERE stream_id = $1"#)
        .bind(stream_id)
        .bind(next_id as i64)
        .execute(&mut **tx)
        .await?;

    Ok(())
}
