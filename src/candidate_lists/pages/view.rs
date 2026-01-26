use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, Context, HtmlTemplate,
    candidate_lists::{CandidateList, FullCandidateList, pages::ViewCandidateListPath},
    filters,
};

#[derive(Template)]
#[template(path = "candidate_lists/view.html")]
struct CandidateListViewTemplate {
    full_list: FullCandidateList,
}

pub async fn view_candidate_list(
    _: ViewCandidateListPath,
    context: Context,
    full_list: FullCandidateList,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        CandidateListViewTemplate { full_list },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use sqlx::PgPool;

    use crate::{
        Context,
        candidate_lists::{self, CandidateListId},
        persons::{self, PersonId},
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };

    #[sqlx::test]
    async fn view_candidate_list_renders_candidates(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person).await?;
        candidate_lists::update_candidate_list_order(&pool, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");

        let response = view_candidate_list(
            ViewCandidateListPath { list_id },
            Context::new_test(pool.clone()).await,
            full_list,
        )
        .await
        .unwrap()
        .into_response();

        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));
        assert!(body.contains(&list.add_candidate_path()));

        Ok(())
    }
}
