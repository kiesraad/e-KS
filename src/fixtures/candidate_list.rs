use chrono::Utc;
use sqlx::PgConnection;

use crate::{
    AppError, ElectionConfig,
    candidate_lists::{self, CandidateList, CandidateListId},
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

pub async fn load(conn: &mut PgConnection) -> Result<(), AppError> {
    let electoral_districts = ElectionConfig::EK2027.electoral_districts().to_vec();

    let persons = persons::repository::list_all_persons(conn).await?;

    let person_ids = collect_person_ids(persons);

    let candidate_list = CandidateList {
        id: CandidateListId::new(),
        electoral_districts,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let candidate_list =
        candidate_lists::repository::create_candidate_list(conn, &candidate_list).await?;

    // Persist the ordered set of persons to ensure deterministic candidate positions.
    candidate_lists::repository::update_candidate_list_order(conn, candidate_list.id, &person_ids)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn test_load(pool: PgPool) {
        crate::fixtures::persons::load(&mut pool.acquire().await.unwrap())
            .await
            .unwrap();
        let mut conn = pool.acquire().await.unwrap();
        load(&mut conn).await.unwrap();

        let lists = candidate_lists::repository::list_candidate_list_with_count(&mut conn)
            .await
            .unwrap();

        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].person_count, FIXTURE_CANDIDATE_LIST_SIZE as i64);
    }
}
