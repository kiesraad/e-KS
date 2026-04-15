use super::AuthorisedAgentsPath;
use crate::{
    AppError, Context, HtmlTemplate, RequestCtx,
    authorised_agents::AuthorisedAgent,
    filters,
    list_submitters::ListSubmitter,
    political_groups::{PoliticalGroup, PoliticalGroupSteps},
};
use askama::Template;
use axum::response::IntoResponse;

#[derive(Template)]
#[template(path = "authorised_agents/pages/view.html")]
struct AuthorisedAgentsTemplate {
    authorised_agents: Vec<AuthorisedAgent>,
    steps: PoliticalGroupSteps,
}

pub async fn list_authorised_agents(
    _: AuthorisedAgentsPath,
    ctx: RequestCtx,
) -> Result<impl IntoResponse, AppError> {
    let steps = PoliticalGroupSteps::new(&ctx.store)?;
    Ok(HtmlTemplate(
        AuthorisedAgentsTemplate {
            authorised_agents: steps.authorised_agents.clone(),
            steps,
        },
        ctx.context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, QueryParamState,
        authorised_agents::AuthorisedAgentId,
        test_utils::{response_body_string, sample_authorised_agent},
    };
    use axum::{http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn list_authorised_agents_shows_created_agent() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);
        authorised_agent.create(&store).await?;

        let response = list_authorised_agents(
            AuthorisedAgentsPath {},
            RequestCtx {
                context: Context::new_test_without_db(),
                store: store.clone(),
                query: QueryParamState::default(),
            },
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
        let store = AppStore::new_for_test();

        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);
        authorised_agent.create(&store).await?;

        let response = list_authorised_agents(
            AuthorisedAgentsPath {},
            RequestCtx {
                context: Context::new_test_without_db(),
                store: store.clone(),
                query: QueryParamState::default(),
            },
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
