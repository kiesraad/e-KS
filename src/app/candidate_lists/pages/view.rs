use askama::Template;
use axum::{extract::Query, response::IntoResponse};

use crate::{
    AppError, AppStore, Context, ElectoralDistrict, HtmlTemplate, QueryParamState,
    candidate_lists::{CandidateList, FullCandidateList, pages::ViewCandidateListPath},
    common::HasSeverity,
    filters,
};

#[derive(Template)]
#[template(path = "app/candidate_lists/pages/view.html")]
struct CandidateListViewTemplate {
    full_list: FullCandidateList,
    duplicate_districts: Vec<ElectoralDistrict>,
    max_candidates_reached: bool,
    import_capped: bool,
}

pub async fn view_candidate_list(
    _: ViewCandidateListPath,
    context: Context,
    full_list: FullCandidateList,
    store: AppStore,
    Query(query): Query<QueryParamState>,
) -> Result<impl IntoResponse, AppError> {
    let duplicate_districts = full_list.list.duplicate_districts(&store);

    Ok(HtmlTemplate(
        CandidateListViewTemplate {
            full_list,
            duplicate_districts,
            max_candidates_reached: query.is_max_candidates_reached(),
            import_capped: query.is_import_capped(),
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context,
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn view_candidate_list_renders_candidates() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        person.create(&store).await?;
        list.clone().update_order(&store, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");

        let response = view_candidate_list(
            ViewCandidateListPath { list_id },
            Context::new_test_without_db(),
            full_list,
            store,
            Query(QueryParamState::default()),
        )
        .await?
        .into_response();

        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));
        assert!(body.contains(&list.add_candidate_path().to_string()));

        Ok(())
    }
}
