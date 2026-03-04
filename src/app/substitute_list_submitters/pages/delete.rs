use axum::response::Response;

use crate::{
    AppError, AppStore, Context, Form, form::EmptyForm, list_submitters::ListSubmitter,
    redirect_success,
};

use super::SubstituteSubmitterDeletePath;

pub async fn delete_substitute_submitter(
    _: SubstituteSubmitterDeletePath,
    context: Context,
    substitute_submitter: ListSubmitter,
    store: AppStore,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.session.csrf_tokens) {
        Err(_) => Err(AppError::CsrfTokenInvalid),
        Ok(_) => {
            substitute_submitter.delete_substitute(&store).await?;

            Ok(redirect_success(ListSubmitter::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum_extra::routing::TypedPath;

    use super::*;
    use crate::QueryParamState;

    use crate::{
        AppError, AppStore, Context, TokenValue, list_submitters::ListSubmitterId,
        test_utils::sample_list_submitter,
    };

    #[tokio::test]
    async fn delete_substitute_submitter_removes_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let sub_submitter_id = ListSubmitterId::new();
        let substitute_submitter = sample_list_submitter(sub_submitter_id);
        substitute_submitter.create_substitute(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;

        let response = delete_substitute_submitter(
            SubstituteSubmitterDeletePath { sub_submitter_id },
            context,
            substitute_submitter,
            store.clone(),
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
            ListSubmitter::list_path()
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
