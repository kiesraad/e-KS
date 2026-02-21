use uuid::Uuid;

use crate::{
    AppError, ElectionConfig, Store,
    candidate_lists::CandidateList,
    pagination::SortDirection,
    persons::{self, Person, PersonId},
};

const FIXTURE_CANDIDATE_LIST_SIZE: usize = 55;

fn collect_person_ids(persons: Vec<Person>) -> Vec<PersonId> {
    persons.into_iter().map(|person| person.id).collect()
}

pub async fn load(store: &Store) -> Result<(), AppError> {
    let electoral_districts = ElectionConfig::EK2027.electoral_districts().to_vec();
    let persons = persons::Person::list(
        store,
        FIXTURE_CANDIDATE_LIST_SIZE,
        0,
        &persons::PersonSort::UpdatedAt,
        &SortDirection::Asc,
    )?;
    let person_ids = collect_person_ids(persons);
    let uuid = Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        b"the_one_and_only_fixture_candidate_list",
    );

    let candidate_list = CandidateList {
        id: uuid.into(),
        electoral_districts,
        candidates: person_ids,
        ..Default::default()
    };

    candidate_list.create(store).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate_lists::CandidateListSummary;
    #[tokio::test]
    async fn test_load() {
        let store = Store::new_for_test().await;
        crate::fixtures::persons::load(&store).await.unwrap();
        load(&store).await.unwrap();

        let lists = CandidateListSummary::list(&store).unwrap();

        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].person_count, FIXTURE_CANDIDATE_LIST_SIZE);
    }
}
