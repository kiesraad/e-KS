use askama::Template;
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
};

use crate::{
    AppError, AppResponse, AppStore, Context, Form, HtmlTemplate, Overlay, QueryParamState,
    common::Problematic,
    filters,
    form::{EmptyForm, FormData},
    list_submitters::ListSubmitter,
};

use super::SubstituteSubmitterDeletePath;

#[derive(Template)]
#[template(path = "substitute_list_submitters/pages/delete.html")]
struct DeleteSubstituteSubmitterTemplate {
    substitute_submitter: ListSubmitter,
    form: FormData<EmptyForm>,
    overlay: Overlay,
}

pub async fn delete_substitute_submitter_confirm(
    _: SubstituteSubmitterDeletePath,
    context: Context,
    substitute_submitter: ListSubmitter,
    Query(query): Query<QueryParamState>,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        DeleteSubstituteSubmitterTemplate {
            form: FormData::new(&context.session.csrf_token),
            substitute_submitter,
            overlay: Overlay::new(&query),
        },
        context,
    ))
}

pub async fn delete_substitute_submitter(
    _: SubstituteSubmitterDeletePath,
    context: Context,
    substitute_submitter: ListSubmitter,
    store: AppStore,
    Query(query): Query<QueryParamState>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.session.csrf_token) {
        Err(_) => Err(AppError::CsrfTokenInvalid),
        Ok(_) => {
            substitute_submitter.delete_substitute(&store).await?;

            Ok(query.redirect_or(ListSubmitter::view_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::Query;
    use axum_extra::routing::TypedPath;

    use super::*;
    use crate::QueryParamState;

    use crate::{
        AppError, AppStore, Context, TokenValue,
        list_submitters::ListSubmitterId,
        test_utils::{response_body_string, sample_list_submitter},
    };

    #[tokio::test]
    async fn delete_substitute_submitter_confirm_contains_delete_button() -> Result<(), AppError> {
        let sub_submitter_id = ListSubmitterId::new();
        let substitute_submitter = sample_list_submitter(sub_submitter_id);

        let response = delete_substitute_submitter_confirm(
            SubstituteSubmitterDeletePath { sub_submitter_id },
            Context::new_test_without_db(),
            substitute_submitter.clone(),
            Query(QueryParamState::default()),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&substitute_submitter.substitute_delete_path().to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn delete_substitute_submitter_removes_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let sub_submitter_id = ListSubmitterId::new();
        let substitute_submitter = sample_list_submitter(sub_submitter_id);
        substitute_submitter.create_substitute(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();

        let response = delete_substitute_submitter(
            SubstituteSubmitterDeletePath { sub_submitter_id },
            context,
            substitute_submitter.clone(),
            store.clone(),
            Query(QueryParamState::default()),
            Form(EmptyForm::new(csrf_token)),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(
            location,
            ListSubmitter::view_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let submitters = store.get_substitute_submitters();
        assert!(submitters.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn delete_substitute_submitter_invalid_csrf_error_page() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let sub_submitter_id = ListSubmitterId::new();
        let substitute_submitter = sample_list_submitter(sub_submitter_id);
        substitute_submitter.create_substitute(&store).await?;

        let context = Context::new_test_without_db();

        let response = delete_substitute_submitter(
            SubstituteSubmitterDeletePath { sub_submitter_id },
            context,
            substitute_submitter.clone(),
            store.clone(),
            Query(QueryParamState::default()),
            Form(EmptyForm::new(TokenValue("invalid".to_string()))),
        )
        .await
        .unwrap_err();

        assert!(matches!(response, AppError::CsrfTokenInvalid));

        let submitters = store.get_substitute_submitters();
        assert_eq!(submitters.len(), 1);

        Ok(())
    }
}
