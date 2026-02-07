use super::ListSubmittersPath;
use crate::{
    AppError, AppStore, Context, HtmlTemplate, authorised_agents::AuthorisedAgent, filters,
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
    let political_group = store.get_political_group()?;
    let authorised_agents = store.get_authorised_agents()?;
    let list_submitters = store.get_list_submitters()?;
    let substitute_submitters = store.get_substitute_submitters()?;
    let steps = PoliticalGroupSteps::new(
        &political_group,
        &authorised_agents,
        &list_submitters,
        &substitute_submitters,
    );

    Ok(HtmlTemplate(
        ListSubmittersTemplate {
            list_submitters,
            substitute_submitters,
            steps,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};
    use sqlx::PgPool;

    use crate::{
        AppError, AppStore, Context,
        list_submitters::ListSubmitterId,
        political_groups::PoliticalGroupId,
        test_utils::{response_body_string, sample_list_submitter, sample_political_group},
    };

    #[sqlx::test]
    async fn list_submitters_shows_created_submitter(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
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
        assert!(body.contains(&list_submitter.last_name));

        Ok(())
    }

    #[sqlx::test]
    async fn list_submitters_shows_edit_link(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
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
