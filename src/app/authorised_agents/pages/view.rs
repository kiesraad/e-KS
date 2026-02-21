use super::AuthorisedAgentsPath;
use crate::{
    AppError, Context, HtmlTemplate, Store,
    authorised_agents::AuthorisedAgent,
    filters,
    list_submitters::ListSubmitter,
    political_groups::{PoliticalGroup, PoliticalGroupSteps},
};
use askama::Template;
use axum::{extract::State, response::IntoResponse};

#[derive(Template)]
#[template(path = "authorised_agents/pages/view.html")]
struct AuthorisedAgentsTemplate {
    authorised_agents: Vec<AuthorisedAgent>,
    steps: PoliticalGroupSteps,
}

pub async fn list_authorised_agents(
    _: AuthorisedAgentsPath,
    context: Context,
    State(store): State<Store>,
) -> Result<impl IntoResponse, AppError> {
    let steps = PoliticalGroupSteps::new(store.clone())?;
    Ok(HtmlTemplate(
        AuthorisedAgentsTemplate {
            authorised_agents: steps.authorised_agents.clone(),
            steps,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, Context, Store,
        authorised_agents::AuthorisedAgentId,
        political_groups::PoliticalGroupId,
        test_utils::{response_body_string, sample_authorised_agent, sample_political_group},
    };
    use axum::{http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn list_authorised_agents_shows_created_agent() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store).await?;

        let response = list_authorised_agents(
            AuthorisedAgentsPath {},
            Context::new_test_without_db(),
            State(store.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(authorised_agent.name.last_name.as_str()));

        Ok(())
    }

    #[tokio::test]
    async fn list_authorised_agents_shows_edit_link() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store).await?;

        let response = list_authorised_agents(
            AuthorisedAgentsPath {},
            Context::new_test_without_db(),
            State(store.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&authorised_agent.update_path().to_string()));

        Ok(())
    }
}
