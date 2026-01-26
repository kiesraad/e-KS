use crate::{
    AppError,
    candidate_lists::{self, CandidateList, pages::CandidateListReorderPath},
    persons::PersonId,
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct CandidateListReorderPayload {
    pub person_ids: Vec<PersonId>,
}

pub async fn reorder_candidate_list(
    _: CandidateListReorderPath,
    candidate_list: CandidateList,
    State(pool): State<PgPool>,
    Json(payload): Json<CandidateListReorderPayload>,
) -> Result<impl IntoResponse, AppError> {
    candidate_lists::update_candidate_list_order(&pool, candidate_list.id, &payload.person_ids)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    use crate::{
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

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person_a).await?;
        persons::create_person(&pool, &person_b).await?;
        candidate_lists::update_candidate_list_order(&pool, list_id, &[person_a.id, person_b.id])
            .await?;

        let response = reorder_candidate_list(
            CandidateListReorderPath { list_id },
            list.clone(),
            State(pool.clone()),
            Json(CandidateListReorderPayload {
                person_ids: vec![person_b.id, person_a.id],
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, person_b.id);
        assert_eq!(full_list.candidates[1].person.id, person_a.id);

        Ok(())
    }
}
