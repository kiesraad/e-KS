use super::ListSubmittersPath;
use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    authorised_agents::AuthorisedAgent,
    filters,
    list_submitters::ListSubmitter,
    political_groups::{PoliticalGroup, PoliticalGroupSteps},
    substitute_list_submitters::SubstituteSubmitter,
};
use askama::Template;
use axum::{extract::State, response::IntoResponse};

#[derive(Template)]
#[template(path = "political_groups/submitters.html")]
struct ListSubmittersTemplate {
    list_submitters: Vec<ListSubmitter>,
    substitute_submitters: Vec<SubstituteSubmitter>,
    steps: PoliticalGroupSteps,
}

pub async fn list_submitters(
    _: ListSubmittersPath,
    context: Context,
    State(store): State<AppStore>,
) -> Result<impl IntoResponse, AppError> {
    let steps = PoliticalGroupSteps::new(store.clone())?;

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
        political_groups::PoliticalGroupId,
        test_utils::{response_body_string, sample_list_submitter, sample_political_group},
    };
    use axum::{http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn list_submitters_shows_created_submitter() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_group.create(&store).await?;
        list_submitter.create(&store).await?;

        let response = list_submitters(
            ListSubmittersPath {},
            Context::new_test_without_db(),
            State(store),
        )
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
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_group.create(&store).await?;
        list_submitter.create(&store).await?;

        let response = list_submitters(
            ListSubmittersPath {},
            Context::new_test_without_db(),
            State(store),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list_submitter.update_path()));

        Ok(())
    }
}
