use uuid::Uuid;

use crate::{
    AppError, AppStore,
    candidate_lists::CandidateList,
    pagination::SortDirection,
    persons::{self, Person, PersonId},
};

const FIXTURE_CANDIDATE_LIST_SIZE: usize = 55;

fn collect_person_ids(persons: Vec<Person>) -> Vec<PersonId> {
    persons.into_iter().map(|person| person.id).collect()
}

pub async fn load(store: &AppStore) -> Result<(), AppError> {
    let election = store.get_election();

    let persons = persons::Person::list(
        store,
        FIXTURE_CANDIDATE_LIST_SIZE,
        0,
        &persons::PersonSort::UpdatedAt,
        &SortDirection::Asc,
    )?;
    let valid_persons = persons::Person::list(
        store,
        1000,
        0,
        &persons::PersonSort::UpdatedAt,
        &SortDirection::Asc,
    )?
    .into_iter()
    .filter(|p| p.is_complete())
    .take(FIXTURE_CANDIDATE_LIST_SIZE)
    .collect::<Vec<_>>();
    let person_ids = collect_person_ids(persons);
    let valid_person_ids = collect_person_ids(valid_persons);

    let candidate_list = CandidateList {
        id: Uuid::new_v5(
            &Uuid::NAMESPACE_OID,
            b"the_one_and_only_fixture_candidate_list",
        )
        .into(),
        electoral_districts: vec![election.electoral_districts()[0]],
        candidates: person_ids,
        ..Default::default()
    };

    candidate_list.create(store).await?;

    if election.has_only_one_district() {
        return Ok(());
    }

    let second_districts: Vec<_> = CandidateList::available_districts(store, &election)
        .into_iter()
        .take(2)
        .collect();

    CandidateList {
        id: Uuid::new_v5(&Uuid::NAMESPACE_OID, b"the_second_fixture_candidate_list").into(),
        electoral_districts: second_districts,
        candidates: valid_person_ids.clone(),
        ..candidate_list.clone()
    }
    .create(store)
    .await?;

    let remaining: Vec<_> = CandidateList::available_districts(store, &election)
        .into_iter()
        .take(4)
        .collect();

    if remaining.is_empty() {
        return Ok(());
    }

    CandidateList {
        id: Uuid::new_v5(&Uuid::NAMESPACE_OID, b"the_third_fixture_candidate_list").into(),
        electoral_districts: remaining,
        candidates: valid_person_ids[..FIXTURE_CANDIDATE_LIST_SIZE / 2].to_vec(),
        ..candidate_list
    }
    .create(store)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ElectionConfig,
        candidate_lists::CandidateListSummary,
        core::election::{Province, WaterCouncil},
    };
    #[tokio::test]
    async fn test_load() {
        let store = AppStore::new_for_test();
        crate::fixtures::persons::load(&store).await.unwrap();
        load(&store).await.unwrap();

        let lists = CandidateListSummary::list(&store);

        assert_eq!(lists.len(), 3);
        assert_eq!(lists[0].person_count, FIXTURE_CANDIDATE_LIST_SIZE);
        for list in &lists {
            for district in &list.list.electoral_districts {
                assert!(
                    ElectionConfig::EK27
                        .electoral_districts()
                        .contains(district)
                );
            }
        }
    }

    #[tokio::test]
    async fn test_load_single_district_election() {
        let store =
            AppStore::new_for_test_with_election(ElectionConfig::WS27(WaterCouncil::Rivierenland));
        crate::fixtures::persons::load(&store).await.unwrap();
        load(&store).await.unwrap();

        let lists = CandidateListSummary::list(&store);

        assert_eq!(lists.len(), 1);
        assert_eq!(
            lists[0].list.electoral_districts,
            ElectionConfig::WS27(WaterCouncil::Rivierenland)
                .electoral_districts()
                .to_vec()
        );
    }

    #[tokio::test]
    async fn test_load_two_district_election() {
        let election = ElectionConfig::PS27(Province::GE);
        let store = AppStore::new_for_test_with_election(election);
        crate::fixtures::persons::load(&store).await.unwrap();
        load(&store).await.unwrap();

        let lists = CandidateListSummary::list(&store);

        assert_eq!(lists.len(), 2);
        for list in &lists {
            for district in &list.list.electoral_districts {
                assert!(election.electoral_districts().contains(district));
            }
        }
    }
}
