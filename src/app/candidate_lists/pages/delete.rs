use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, AppResponse, AppStore, Context, Form, HtmlTemplate,
    candidate_lists::{CandidateList, pages::CandidateListsDeletePath},
    filters,
    form::{EmptyForm, FormData},
    redirect_success,
};

#[derive(Template)]
#[template(path = "candidate_lists/pages/delete.html")]
struct DeleteCandidateListTemplate {
    candidate_list: CandidateList,
    form: FormData<EmptyForm>,
}

pub async fn delete_candidate_list_confirm(
    _: CandidateListsDeletePath,
    context: Context,
    candidate_list: CandidateList,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        DeleteCandidateListTemplate {
            form: FormData::new(&context.session.csrf_token),
            candidate_list,
        },
        context,
    ))
}

pub async fn delete_candidate_list(
    _: CandidateListsDeletePath,
    context: Context,
    candidate_list: CandidateList,
    store: AppStore,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.session.csrf_token) {
        Err(_) => Err(AppError::CsrfTokenInvalid),
        Ok(_) => {
            candidate_list.delete(&store).await?;

            Ok(redirect_success(CandidateList::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, ElectoralDistrict, Form, QueryParamState, TokenValue,
        candidate_lists::CandidateListSummary, test_utils::response_body_string,
    };
    use axum::http::{StatusCode, header};
    use axum_extra::routing::TypedPath;

    #[tokio::test]
    async fn delete_candidate_list_confirm_contains_delete_button() -> Result<(), AppError> {
        let candidate_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            ..Default::default()
        };

        let response = delete_candidate_list_confirm(
            CandidateListsDeletePath {
                list_id: candidate_list.id,
            },
            Context::new_test_without_db(),
            candidate_list.clone(),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&candidate_list.delete_path().to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn delete_candidate_list_and_redirect() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();
        let candidate_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            ..Default::default()
        };
        candidate_list.create(&store).await?;

        let response = delete_candidate_list(
            CandidateListsDeletePath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            store.clone(),
            Form(EmptyForm { csrf_token }),
        )
        .await?;

        // verify redirect
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");

        assert_eq!(
            location,
            CandidateList::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        // verify deletion (i.e. no lists in database left)
        let lists = CandidateListSummary::list(&store);
        assert_eq!(lists.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn delete_candidate_invalid_csrf_error_page() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let context = Context::new_test_without_db();
        let csrf_token = TokenValue("invalid".to_string());
        let candidate_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            ..Default::default()
        };
        candidate_list.create(&store).await?;

        let response = delete_candidate_list(
            CandidateListsDeletePath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            store.clone(),
            Form(EmptyForm { csrf_token }),
        )
        .await
        .unwrap_err();

        assert!(matches!(response, AppError::CsrfTokenInvalid));

        // verify deletion didn't go through (i.e. still 1 list in database left)
        let lists = CandidateListSummary::list(&store);
        assert_eq!(lists.len(), 1);

        Ok(())
    }
}
