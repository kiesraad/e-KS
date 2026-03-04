use super::ListSubmittersPath;
use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    authorised_agents::AuthorisedAgent,
    filters,
    list_submitters::ListSubmitter,
    political_groups::{PoliticalGroup, PoliticalGroupSteps},
};
use askama::Template;
use axum::response::IntoResponse;

#[derive(Template)]
#[template(path = "list_submitters/pages/view.html")]
struct ListSubmittersTemplate {
    list_submitters: Vec<ListSubmitter>,
    substitute_submitters: Vec<ListSubmitter>,
    steps: PoliticalGroupSteps,
}

pub async fn list_submitters(
    _: ListSubmittersPath,
    context: Context,
    store: AppStore,
) -> Result<impl IntoResponse, AppError> {
    let steps = PoliticalGroupSteps::new(&store)?;

    Ok(HtmlTemplate(
        ListSubmittersTemplate {
            list_submitters: steps.list_submitters.clone(),
            substitute_submitters: steps.substitute_submitters.clone(),
            steps,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context,
        list_submitters::ListSubmitterId,
        test_utils::{response_body_string, sample_list_submitter},
    };
    use axum::{http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn list_submitters_shows_created_submitter() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);
        list_submitter.create(&store).await?;

        let response =
            list_submitters(ListSubmittersPath {}, Context::new_test_without_db(), store)
                .await
                .unwrap()
                .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(list_submitter.name.last_name.as_str()));

        Ok(())
    }

    #[tokio::test]
    async fn list_submitters_shows_edit_link() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);
        list_submitter.create(&store).await?;

        let response =
            list_submitters(ListSubmittersPath {}, Context::new_test_without_db(), store)
                .await
                .unwrap()
                .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list_submitter.update_path().to_string()));

        Ok(())
    }
}
