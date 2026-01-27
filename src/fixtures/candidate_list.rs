use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AppError, ElectionConfig,
    candidate_lists::{self, CandidateList},
    persons::{self, Person, PersonId},
};

const FIXTURE_CANDIDATE_LIST_SIZE: usize = 55;

fn collect_person_ids(persons: Vec<Person>) -> Vec<PersonId> {
    persons
        .into_iter()
        .map(|person| person.id)
        .take(FIXTURE_CANDIDATE_LIST_SIZE)
        .collect()
}

pub async fn load(db: &PgPool) -> Result<(), AppError> {
    let electoral_districts = ElectionConfig::EK2027.electoral_districts().to_vec();
    let persons = persons::list_all_persons(db).await?;
    let person_ids = collect_person_ids(persons);
    let uuid = Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        b"the_one_and_only_fixture_candidate_list",
    );

    let candidate_list = CandidateList {
        id: uuid.into(),
        electoral_districts,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let candidate_list = candidate_lists::create_candidate_list(db, &candidate_list).await?;

    // Persist the ordered set of persons to ensure deterministic candidate positions.
    candidate_lists::update_candidate_list_order(db, candidate_list.id, &person_ids).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn test_load(pool: PgPool) {
        crate::fixtures::persons::load(&pool).await.unwrap();
        load(&pool).await.unwrap();

        let lists = candidate_lists::list_candidate_list_summary(&pool)
            .await
            .unwrap();

        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].person_count, FIXTURE_CANDIDATE_LIST_SIZE as i64);
    }
}
