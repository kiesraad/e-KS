use crate::{AppError, AppStore};

mod candidate_list;
mod persons;
mod political_groups;

pub async fn load(store: &AppStore) -> Result<(), AppError> {
    store.clear().await?;
    persons::load(store).await?;
    candidate_list::load(store).await?;
    political_groups::load(store).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{AppStore, fixtures::load};
    use sqlx::PgPool;

    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test]
    async fn test_load_all_fixtures(pool: PgPool) {
        let store = AppStore::new(pool);
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
