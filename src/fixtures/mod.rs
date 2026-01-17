use sqlx::{PgConnection, PgPool};

use crate::AppError;

mod candidate_list;
mod persons;

pub async fn load(pool: PgPool) -> Result<(), AppError> {
    let mut conn = pool.acquire().await?;

    clear_database(&mut conn).await?;
    persons::load(&mut conn).await?;
    candidate_list::load(&mut conn).await?;

    Ok(())
}

async fn clear_database(conn: &mut PgConnection) -> Result<(), AppError> {
    sqlx::query!(
        "
        TRUNCATE TABLE candidate_lists_persons, candidate_lists, persons
        RESTART IDENTITY CASCADE
        "
    )
    .execute(conn)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::fixtures::load;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_load_all_fixtures(pool: PgPool) {
        load(pool.clone()).await.unwrap();
        let mut conn = pool.acquire().await.unwrap();
        let persons = crate::persons::repository::list_persons(
            &mut conn,
            50,
            0,
            &crate::persons::PersonSort::LastName,
            &crate::pagination::SortDirection::Asc,
        )
        .await;

        assert!(persons.is_ok());
        assert_eq!(persons.unwrap().len(), 50);
    }
}
