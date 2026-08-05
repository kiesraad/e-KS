use crate::{AppError, PgStore, structs::common::DisplayName};

mod candidate_list;
mod persons;
mod political_groups;

pub async fn load(store: &PgStore, display_name: Option<DisplayName>) -> Result<(), AppError> {
    let person_count = store.get_person_count();
    let candidate_list_count = store.get_candidate_list_count();

    if person_count > 0 && candidate_list_count > 0 {
        tracing::warn!("Skip loading fixtures, store not empty");

        return Ok(());
    }

    persons::load(store).await?;
    candidate_list::load(store).await?;
    political_groups::load(store, display_name).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{PgStore, fixtures::load};

    #[tokio::test]
    async fn test_load_all_fixtures() {
        let store = PgStore::new_for_test();
        load(&store, None).await.unwrap();
        let persons = crate::structs::persons::Person::list(
            &store,
            50,
            0,
            &crate::structs::persons::PersonSort::LastName,
            &crate::pagination::SortDirection::Asc,
        )
        .unwrap();

        assert_eq!(persons.len(), 50);
    }
}
