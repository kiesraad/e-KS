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
        AppEvent, AppStore, Context,
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };

    #[sqlx::test]
    async fn view_candidate_list_renders_candidates(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;
        CandidateList::update_order(&store, list_id, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let response = view_candidate_list(
            ViewCandidateListPath { list_id },
            Context::new_test_without_db(),
            full_list,
        )
        .await?
        .into_response();

        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));
        assert!(body.contains(&list.add_candidate_path()));

        Ok(())
    }
}
