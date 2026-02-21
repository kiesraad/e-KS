use crate::{AppError, AppEvent, AppStore, constants::DEFAULT_STREAM_ID};

#[derive(Debug, sqlx::FromRow)]
pub struct DatabaseEvent {
    pub event_id: i64,
    pub payload: serde_json::Value,
    // pub created_at: chrono::DateTime<chrono::Utc>,
}

// #[cfg(feature = "migrations")]
pub async fn migrate(pool: &sqlx::PgPool) -> Result<(), AppError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS streams (
          stream_id UUID PRIMARY KEY,
          last_event_id BIGINT NOT NULL
        )
        "#,
    )
    .execute(pool)
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
    .execute(pool)
    .await?;

    sqlx::query(
        r#"INSERT INTO streams (stream_id, last_event_id)
        VALUES ($1, 0)
        ON CONFLICT (stream_id) DO NOTHING"#,
    )
    .bind(crate::constants::DEFAULT_STREAM_ID)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn load_from_database(store: &AppStore, pool: &sqlx::PgPool) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    if let Err(err) = catch_up(store, &mut tx).await {
        tx.rollback().await?;
        return Err(err);
    }

    Ok(())
}

pub async fn update_in_database(
    store: &AppStore,
    pool: &sqlx::PgPool,
    event: AppEvent,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    let last_id = match catch_up(store, &mut tx).await {
        Ok(id) => id,
        Err(err) => {
            tx.rollback().await?;
            return Err(err);
        }
    };

    let next_id = last_id + 1;

    if let Err(err) = append_once(next_id, &event, &mut tx).await {
        tx.rollback().await?;
        return Err(err);
    }

    tx.commit().await?;

    let mut data = store.data.write();

    if data.last_event_id >= next_id {
        // This can happen if another instance of the application processed events concurrently
        // and updated the store before this instance could acquire the write lock. In that case,
        // the store is already up-to-date and we can skip applying the event again.
        return Ok(());
    }

    AppStore::apply(event, &mut data);
    data.last_event_id = next_id;

    Ok(())
}

async fn catch_up(
    store: &AppStore,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<usize, AppError> {
    let last_id: usize = store.get_last_event_id()?;

    let stream_last_id: i64 = sqlx::query_scalar(
        r#"SELECT last_event_id
        FROM streams
        WHERE stream_id = $1
        FOR UPDATE"#,
    )
    .bind(DEFAULT_STREAM_ID)
    .fetch_one(&mut **tx)
    .await?;

    let missing: Vec<DatabaseEvent> = sqlx::query_as::<_, DatabaseEvent>(
        r#"
        SELECT event_id, payload
        FROM events
        WHERE stream_id = $1 AND event_id > $2
        ORDER BY event_id ASC
        "#,
    )
    .bind(DEFAULT_STREAM_ID)
    .bind(last_id as i64)
    .fetch_all(&mut **tx)
    .await?;

    let mut data = store.data.write();

    for event in missing {
        if data.last_event_id >= event.event_id as usize {
            // This can happen if another instance of the application processed events concurrently
            // and updated the store before this instance could acquire the write lock. In that case,
            // the store is already up-to-date and we can skip applying the event again.
            continue;
        }

        match serde_json::from_value::<AppEvent>(event.payload) {
            Ok(ev) => {
                AppStore::apply(ev, &mut data);
                data.last_event_id = event.event_id as usize;
            }
            Err(e) => {
                tracing::error!("Failed to deserialize event: {e:?}");
                continue;
            }
        }
    }

    Ok(stream_last_id as usize)
}

async fn append_once(
    next_id: usize,
    event: &AppEvent,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), AppError> {
    let new_payload = serde_json::to_value(event).map_err(|_| AppError::InternalServerError)?;

    sqlx::query(
        r#"INSERT INTO events (stream_id, event_id, created_at, payload)
        VALUES ($1, $2, $3, $4)"#,
    )
    .bind(DEFAULT_STREAM_ID)
    .bind(next_id as i64)
    .bind(chrono::Utc::now())
    .bind(new_payload)
    .execute(&mut **tx)
    .await?;

    sqlx::query(r#"UPDATE streams SET last_event_id = $2 WHERE stream_id = $1"#)
        .bind(DEFAULT_STREAM_ID)
        .bind(next_id as i64)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{persons::PersonId, test_utils::sample_person};
    use chrono::Utc;
    use sqlx::PgPool;

    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test(migrations = false)]
    async fn update_persists_and_load_replays(pool: PgPool) -> Result<(), AppError> {
        #[cfg(feature = "migrations")]
        migrate(&pool).await?;

        let store = AppStore::new_with_pool(pool.clone()).await.unwrap();
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let loaded = store.get_person(person_id)?;
        assert_eq!(loaded.id, person_id);

        let fresh_store = AppStore::new_with_pool(pool).await.unwrap();
        fresh_store.load().await?;

        let reloaded = fresh_store.get_person(person_id)?;
        assert_eq!(reloaded.id, person_id);

        Ok(())
    }

    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test(migrations = false)]
    async fn load_skips_invalid_payloads(pool: PgPool) -> Result<(), AppError> {
        // #[cfg(feature = "migrations")]
        migrate(&pool).await?;

        let store = AppStore::new_with_pool(pool.clone()).await.unwrap();
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let invalid_payload = serde_json::json!({"not": "an app event"});
        sqlx::query(
            r#"INSERT INTO events (stream_id, event_id, created_at, payload)
            VALUES ($1, $2, $3, $4)"#,
        )
        .bind(DEFAULT_STREAM_ID)
        .bind(2_i64)
        .bind(Utc::now())
        .bind(invalid_payload)
        .execute(&pool)
        .await?;

        sqlx::query(r#"UPDATE streams SET last_event_id = $2 WHERE stream_id = $1"#)
            .bind(DEFAULT_STREAM_ID)
            .bind(2_i64)
            .execute(&pool)
            .await?;

        let fresh_store = AppStore::new_with_pool(pool).await.unwrap();
        fresh_store.load().await?;

        let reloaded = fresh_store.get_person(person_id)?;
        assert_eq!(reloaded.id, person_id);

        Ok(())
    }
}
