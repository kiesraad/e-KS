use axum::{extract::State, response::Response};

use crate::{
    AppError, AppStore, Context, Form, form::EmptyForm, list_submitters::ListSubmitter,
    redirect_success,
};

use super::ListSubmitterDeletePath;

pub async fn delete_list_submitter(
    _: ListSubmitterDeletePath,
    context: Context,
    submitter: ListSubmitter,
    State(store): State<AppStore>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => Err(AppError::CsrfTokenInvalid),
        Ok(_) => {
            submitter.delete(&store).await?;

            Ok(redirect_success(ListSubmitter::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum_extra::routing::TypedPath;

    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState, TokenValue,
        list_submitters::{ListSubmitter, ListSubmitterId},
        political_groups::PoliticalGroupId,
        test_utils::{sample_list_submitter, sample_political_group},
    };

    #[tokio::test]
    async fn delete_list_submitter_removes_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let submitter = sample_list_submitter(submitter_id);

        political_group.create(&store).await?;
        submitter.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_list_submitter(
            ListSubmitterDeletePath { submitter_id },
            context,
            submitter,
            State(store.clone()),
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

        let submitters = store.get_list_submitters()?;
        assert!(submitters.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn delete_list_submitter_invalid_csrf_error_page() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_group.create(&store).await?;
        list_submitter.create(&store).await?;

        let context = Context::new_test_without_db();

        let response = delete_list_submitter(
            ListSubmitterDeletePath { submitter_id },
            context,
            list_submitter.clone(),
            State(store.clone()),
            Form(EmptyForm::new(TokenValue("invalid".to_string()))),
        )
        .await
        .unwrap_err();

        assert!(matches!(response, AppError::CsrfTokenInvalid));

        let submitters = store.get_list_submitters()?;
        assert_eq!(submitters.len(), 1);

        Ok(())
    }
}
