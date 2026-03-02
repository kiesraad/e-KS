use uuid::Uuid;

use crate::{
    AppError, AppStore, ElectoralDistrict,
    candidate_lists::CandidateList,
    list_submitters::ListSubmitterId,
    pagination::SortDirection,
    persons::{self, Person, PersonId},
    substitute_list_submitters::SubstituteSubmitterId,
};

const FIXTURE_CANDIDATE_LIST_SIZE: usize = 55;

fn collect_person_ids(persons: Vec<Person>) -> Vec<PersonId> {
    persons.into_iter().map(|person| person.id).collect()
}

pub async fn load(store: &AppStore) -> Result<(), AppError> {
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

    let submitter_id: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_list_submitter").into();

    let substitute_submitter_id: SubstituteSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_substitute_submitter_1").into();

    let candidate_list = CandidateList {
        id: Uuid::new_v5(
            &Uuid::NAMESPACE_OID,
            b"the_one_and_only_fixture_candidate_list",
        )
        .into(),
        electoral_districts: vec![ElectoralDistrict::NH],
        candidates: person_ids,
        list_submitter_id: Some(submitter_id),
        substitute_list_submitter_ids: vec![substitute_submitter_id],
        ..Default::default()
    };

    candidate_list.create(store).await?;

    CandidateList {
        id: Uuid::new_v5(&Uuid::NAMESPACE_OID, b"the_second_fixture_candidate_list").into(),
        electoral_districts: vec![ElectoralDistrict::GR, ElectoralDistrict::FR],
        candidates: valid_person_ids.clone(),
        ..candidate_list.clone()
    }
    .create(store)
    .await?;

    CandidateList {
        id: Uuid::new_v5(&Uuid::NAMESPACE_OID, b"the_third_fixture_candidate_list").into(),
        electoral_districts: vec![
            ElectoralDistrict::UT,
            ElectoralDistrict::NB,
            ElectoralDistrict::GE,
            ElectoralDistrict::OV,
        ],
        candidates: valid_person_ids,
        ..candidate_list
    }
    .create(store)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate_lists::CandidateListSummary;
    #[tokio::test]
    async fn test_load() {
        let store = AppStore::new_for_test().await;
        crate::fixtures::persons::load(&store).await.unwrap();
        load(&store).await.unwrap();

        let lists = CandidateListSummary::list(&store).unwrap();

        assert_eq!(lists.len(), 3);
        assert_eq!(lists[0].person_count, FIXTURE_CANDIDATE_LIST_SIZE);
    }
}
