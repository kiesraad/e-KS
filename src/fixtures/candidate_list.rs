use chrono::Utc;
use sqlx::PgConnection;

use crate::{
    AppError, Config,
    candidate_lists::{repository as candidate_list_repository, structs::CandidateList},
    pagination::SortDirection,
    persons::{repository as persons_repository, structs::PersonSort},
};

pub async fn load(conn: &mut PgConnection) -> Result<(), AppError> {
    let config = Config::from_env()?;
    let total_persons = persons_repository::count_persons(conn).await?;
    let electoral_districts = config.get_districts().to_vec();

    let persons = persons_repository::list_persons(
        conn,
        total_persons,
        0,
        &PersonSort::LastName,
        &SortDirection::Asc,
    )
    .await?;

    let person_ids = persons
        .into_iter()
        .map(|person| person.id)
        .take(52)
        .collect::<Vec<_>>();

    let candidate_list = CandidateList {
        id: uuid::Uuid::new_v4(),
        electoral_districts,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let candidate_list =
        candidate_list_repository::create_candidate_list(conn, &candidate_list).await?;

    // Persist the ordered set of persons to ensure deterministic candidate positions.
    let _ = candidate_list_repository::update_candidate_list(conn, &candidate_list.id, &person_ids)
        .await?;

    Ok(())
}
