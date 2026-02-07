use sqlx::PgPool;

use crate::{AppError, AppStore};

mod candidate_list;
mod persons;
mod political_groups;

pub async fn load(store: &AppStore) -> Result<(), AppError> {
    clear_database(&store.pool).await?;
    persons::load(store).await?;
    candidate_list::load(store).await?;
    political_groups::load(store).await?;

    Ok(())
}

async fn clear_database(db: &PgPool) -> Result<(), AppError> {
    sqlx::query("TRUNCATE TABLE streams CASCADE")
        .execute(db)
        .await?;
    sqlx::query("TRUNCATE TABLE events CASCADE")
        .execute(db)
        .await?;

    sqlx::query(
        r#"INSERT INTO streams (stream_id, last_event_id)
        VALUES ($1, 0)
        ON CONFLICT (stream_id) DO NOTHING"#,
    )
    .bind(crate::common::constants::DEFAULT_STREAM_ID)
    .execute(db)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{AppStore, fixtures::load};
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_load_all_fixtures(pool: PgPool) {
        let store = AppStore::new(pool.clone());
        load(&store).await.unwrap();
        let persons = crate::persons::Person::list(
            &store,
            50,
            0,
            &crate::persons::PersonSort::LastName,
            &crate::pagination::SortDirection::Asc,
        )
        .unwrap();

        assert_eq!(persons.len(), 50);
    }
}
