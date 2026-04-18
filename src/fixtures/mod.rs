use crate::{AppError, AppStore, ElectionConfig};

mod candidate_list;
mod persons;
mod political_groups;

pub async fn load(store: &AppStore, election: ElectionConfig) -> Result<(), AppError> {
    let person_count = store.get_person_count();
    let candidate_list_count = store.get_candidate_list_count();

    if person_count > 0 && candidate_list_count > 0 {
        tracing::warn!("Skip loading fixtures, store not empty");

        return Ok(());
    }

    persons::load(store).await?;
    if !election.has_only_one_district() {
        candidate_list::load(store).await?;
    }
    political_groups::load(store).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{AppStore, fixtures::load};

    #[tokio::test]
    async fn test_load_all_fixtures() {
        let store = AppStore::new_for_test();
        load(&store, crate::ElectionConfig::EK27).await.unwrap();
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
