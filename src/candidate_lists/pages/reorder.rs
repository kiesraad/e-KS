use crate::{
    AppError, DbConnection,
    candidate_lists::{self, CandidateList, pages::CandidateListReorderPath},
    persons::PersonId,
};
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CandidateListReorderPayload {
    pub person_ids: Vec<PersonId>,
}

pub async fn reorder_candidate_list(
    _: CandidateListReorderPath,
    candidate_list: CandidateList,
    DbConnection(mut conn): DbConnection,
    Json(payload): Json<CandidateListReorderPayload>,
) -> Result<impl IntoResponse, AppError> {
    candidate_lists::update_candidate_list_order(
        &mut conn,
        candidate_list.id,
        &payload.person_ids,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    use crate::{
        DbConnection,
        candidate_lists::{self, CandidateListId},
        persons::{self, PersonId},
        test_utils::{sample_candidate_list, sample_person_with_last_name},
    };

    #[sqlx::test]
    async fn reorder_candidate_list_updates_positions(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person_a).await?;
        persons::create_person(&mut conn, &person_b).await?;
        candidate_lists::update_candidate_list_order(
            &mut conn,
            list_id,
            &[person_a.id, person_b.id],
        )
        .await?;

        let response = reorder_candidate_list(
            CandidateListReorderPath { list_id },
            list.clone(),
            DbConnection(pool.acquire().await?),
            Json(CandidateListReorderPayload {
                person_ids: vec![person_b.id, person_a.id],
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let mut conn = pool.acquire().await?;
        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, person_b.id);
        assert_eq!(full_list.candidates[1].person.id, person_a.id);

        Ok(())
    }
}
