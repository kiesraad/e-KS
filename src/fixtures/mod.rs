use crate::{AppError, Store};

mod candidate_list;
mod persons;
mod political_groups;

pub async fn load(store: &Store) -> Result<(), AppError> {
    let person_count = store.get_person_count()?;
    let candidate_list_count = store.get_candidate_list_count()?;

    if person_count > 0 && candidate_list_count > 0 {
        tracing::warn!("Skip loading fixtures, store not empty");

        return Ok(());
    }

    persons::load(store).await?;
    candidate_list::load(store).await?;
    political_groups::load(store).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{Store, fixtures::load};

    #[tokio::test]
    async fn test_load_all_fixtures() {
        let store = Store::new_for_test().await;
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
