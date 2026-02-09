use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{AppError, AppStore, Context, form::EmptyForm, list_submitters::ListSubmitter};

use super::ListSubmitterDeletePath;

pub async fn delete_list_submitter(
    _: ListSubmitterDeletePath,
    context: Context,
    submitter: ListSubmitter,
    State(store): State<AppStore>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => Ok(Redirect::to(&submitter.update_path()).into_response()),
        Ok(_) => {
            submitter.delete(&store).await?;

            Ok(Redirect::to(&ListSubmitter::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, TokenValue,
        list_submitters::{ListSubmitter, ListSubmitterId},
        political_groups::PoliticalGroupId,
        test_utils::{sample_list_submitter, sample_political_group},
    };
    use axum_extra::extract::Form;

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
        assert_eq!(location, ListSubmitter::list_path());

        let submitters = store.get_list_submitters()?;
        assert!(submitters.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn delete_list_submitter_invalid_csrf_redirects_to_edit() -> Result<(), AppError> {
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
        .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");

        assert_eq!(location, list_submitter.update_path());

        let submitters = store.get_list_submitters()?;
        assert_eq!(submitters.len(), 1);

        Ok(())
    }
}
