use crate::{AppError, AppEvent, AppStore, constants::DEFAULT_STREAM_ID};

#[derive(Debug, sqlx::FromRow)]
pub struct DatabaseEvent {
    // pub event_id: i64,
    pub payload: serde_json::Value,
    // pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AppStore {
    pub async fn load(&self) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        match self.catch_up(&mut tx).await {
            Ok(_) => {}
            Err(e) => {
                tx.rollback().await?;

                return Err(e);
            }
        }

        Ok(())
    }

    async fn catch_up(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<usize, AppError> {
        let last_id = self.get_last_event_id()?;

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
            SELECT payload
            FROM events
            WHERE stream_id = $1 AND event_id > $2
            ORDER BY event_id ASC
            "#,
        )
        .bind(DEFAULT_STREAM_ID)
        .bind(last_id as i64)
        .fetch_all(&mut **tx)
        .await?;

        let mut data = self.data.write();

        for event in missing {
            let app_event: AppEvent =
                serde_json::from_value(event.payload).map_err(|_| AppError::InternalServerError)?;
            AppStore::apply(&app_event, &mut data);
        }

        Ok(stream_last_id as usize)
    }

    async fn append_once(
        &self,
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

    pub async fn update(&self, event: AppEvent) -> Result<(), AppError> {
        sqlx::query(
            r#"INSERT INTO streams (stream_id, last_event_id)
            VALUES ($1, 0)
            ON CONFLICT (stream_id) DO NOTHING"#,
        )
        .bind(crate::common::constants::DEFAULT_STREAM_ID)
        .execute(&self.pool)
        .await?;

        let mut tx = self.pool.begin().await?;

        let last_id = match self.catch_up(&mut tx).await {
            Ok(id) => id,
            Err(e) => {
                tx.rollback().await?;

                return Err(e);
            }
        };

        let next_id = last_id + 1;

        match self.append_once(next_id, &event, &mut tx).await {
            Ok(_) => {}
            Err(e) => {
                tx.rollback().await?;

                return Err(e);
            }
        }

        tx.commit().await?;

        let mut data = self.data.write();
        AppStore::apply(&event, &mut data);
        data.last_event_id = next_id;

        Ok(())
    }
}
