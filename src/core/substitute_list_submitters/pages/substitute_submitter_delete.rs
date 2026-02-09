use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, form::EmptyForm, list_submitters::ListSubmitter,
    substitute_list_submitters::SubstituteSubmitter,
};

use super::SubstituteSubmitterDeletePath;

pub async fn delete_substitute_submitter(
    _: SubstituteSubmitterDeletePath,
    context: Context,
    substitute_submitter: SubstituteSubmitter,
    State(store): State<AppStore>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => Ok(Redirect::to(&substitute_submitter.update_path()).into_response()),
        Ok(_) => {
            substitute_submitter.delete(&store).await?;

            Ok(Redirect::to(&ListSubmitter::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_extra::extract::Form;

    use crate::{
        AppError, AppStore, Context, TokenValue,
        political_groups::PoliticalGroupId,
        substitute_list_submitters::SubstituteSubmitterId,
        test_utils::{sample_political_group, sample_substitute_submitter},
    };

    #[tokio::test]
    async fn delete_substitute_submitter_removes_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let sub_submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(sub_submitter_id);

        political_group.create(&store).await?;
        substitute_submitter.create(&store).await?;
        political_group.update(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_substitute_submitter(
            SubstituteSubmitterDeletePath { sub_submitter_id },
            context,
            substitute_submitter,
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

        let submitters = store.get_substitute_submitters()?;
        assert!(submitters.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn delete_substitute_submitter_invalid_csrf_redirects_to_edit() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let sub_submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(sub_submitter_id);

        political_group.create(&store).await?;
        substitute_submitter.create(&store).await?;
        political_group.update(&store).await?;

        let context = Context::new_test_without_db();

        let response = delete_substitute_submitter(
            SubstituteSubmitterDeletePath { sub_submitter_id },
            context,
            substitute_submitter.clone(),
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
        assert_eq!(location, substitute_submitter.update_path());

        let submitters = store.get_substitute_submitters()?;
        assert_eq!(submitters.len(), 1);

        Ok(())
    }
}
