use sqlx::PgPool;

use crate::AppError;

mod candidate_list;
mod persons;
mod political_groups;

pub async fn load(pool: &PgPool) -> Result<(), AppError> {
    clear_database(pool).await?;
    persons::load(pool).await?;
    candidate_list::load(pool).await?;
    political_groups::load(pool).await?;

    Ok(())
}

async fn clear_database(db: &PgPool) -> Result<(), AppError> {
    sqlx::query!(
        "
        TRUNCATE TABLE authorised_agents, list_submitters, political_groups,
        candidate_lists_persons, candidate_lists, persons
        RESTART IDENTITY CASCADE
        "
    )
    .execute(db)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::fixtures::load;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_load_all_fixtures(pool: PgPool) {
        load(&pool).await.unwrap();
        let persons = crate::persons::list_persons(
            &pool,
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
