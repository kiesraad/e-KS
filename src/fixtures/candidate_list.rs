use chrono::Utc;
use sqlx::PgConnection;

use crate::{
    AppError, Config,
    candidate_lists::{self, CandidateList},
    pagination::SortDirection,
    persons::{self, PersonSort},
};

pub async fn load(conn: &mut PgConnection) -> Result<(), AppError> {
    let config = Config::from_env()?;
    let total_persons = persons::repository::count_persons(conn).await?;
    let electoral_districts = config.get_districts().to_vec();

    let persons = persons::repository::list_persons(
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
        .take(55)
        .collect::<Vec<_>>();

    let candidate_list = CandidateList {
        id: uuid::Uuid::new_v4(),
        electoral_districts,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let candidate_list =
        candidate_lists::repository::create_candidate_list(conn, &candidate_list).await?;

    // Persist the ordered set of persons to ensure deterministic candidate positions.
    candidate_lists::repository::update_candidate_list_order(conn, &candidate_list.id, &person_ids)
        .await?;

    Ok(())
}
