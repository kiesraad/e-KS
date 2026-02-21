use crate::{
    AppError, Store,
    candidate_lists::{CandidateList, pages::CandidateListReorderPath},
    persons::PersonId,
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CandidateListReorderPayload {
    pub person_ids: Vec<PersonId>,
}

pub async fn reorder_candidate_list(
    _: CandidateListReorderPath,
    mut candidate_list: CandidateList,
    State(store): State<Store>,
    Json(payload): Json<CandidateListReorderPayload>,
) -> Result<impl IntoResponse, AppError> {
    candidate_list
        .update_order(&store, &payload.person_ids)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Store,
        candidate_lists::{CandidateListId, FullCandidateList},
        persons::PersonId,
        test_utils::{sample_candidate_list, sample_person_with_last_name},
    };

    #[tokio::test]
    async fn reorder_candidate_list_updates_positions() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        person_a.create(&store).await?;
        person_b.create(&store).await?;
        list.clone()
            .update_order(&store, &[person_a.id, person_b.id])
            .await?;

        let response = reorder_candidate_list(
            CandidateListReorderPath { list_id },
            list.clone(),
            State(store.clone()),
            Json(CandidateListReorderPayload {
                person_ids: vec![person_b.id, person_a.id],
            }),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, person_b.id);
        assert_eq!(full_list.candidates[1].person.id, person_a.id);

        Ok(())
    }
}
