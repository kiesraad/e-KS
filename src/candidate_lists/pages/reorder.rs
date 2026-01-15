use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppError, Context, DbConnection, candidate_lists};

use super::{CandidateListReorderPath, load_candidate_list};

#[derive(Deserialize)]
pub(crate) struct CandidateListReorderPayload {
    pub person_ids: Vec<Uuid>,
}

pub(crate) async fn reorder_candidate_list(
    CandidateListReorderPath { id }: CandidateListReorderPath,
    context: Context,
    DbConnection(mut conn): DbConnection,
    Json(payload): Json<CandidateListReorderPayload>,
) -> Result<impl IntoResponse, AppError> {
    load_candidate_list(&mut conn, &id, context.locale).await?;
    candidate_lists::repository::update_candidate_list_order(&mut conn, &id, &payload.person_ids)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        Context, DbConnection, Locale, candidate_lists, persons,
        test_utils::{sample_candidate_list, sample_person_with_last_name},
    };

    #[sqlx::test]
    async fn reorder_candidate_list_updates_positions(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(Uuid::new_v4(), "Jansen");
        let person_b = sample_person_with_last_name(Uuid::new_v4(), "Bakker");

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person_a).await?;
        persons::repository::create_person(&mut conn, &person_b).await?;
        candidate_lists::repository::update_candidate_list_order(
            &mut conn,
            &list_id,
            &[person_a.id, person_b.id],
        )
        .await?;

        let response = reorder_candidate_list(
            CandidateListReorderPath { id: list_id },
            Context::new(Locale::En),
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
        let full_list = load_candidate_list(&mut conn, &list_id, Locale::En)
            .await
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, person_b.id);
        assert_eq!(full_list.candidates[1].person.id, person_a.id);

        Ok(())
    }
}
