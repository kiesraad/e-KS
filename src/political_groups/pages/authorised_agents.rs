use super::AuthorisedAgentsPath;
use crate::{
    AppError, Context, HtmlTemplate, filters,
    political_groups::{self, AuthorisedAgent, ListSubmitter, PoliticalGroup, SubstituteSubmitter},
};
use askama::Template;
use axum::{extract::State, response::IntoResponse};
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "political_groups/authorised_agents.html")]
struct AuthorisedAgentsTemplate {
    authorised_agents: Vec<AuthorisedAgent>,
}

pub async fn list_authorised_agents(
    _: AuthorisedAgentsPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AppError> {
    let authorised_agents =
        political_groups::get_authorised_agents(&pool, political_group.id).await?;

    Ok(HtmlTemplate(
        AuthorisedAgentsTemplate { authorised_agents },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};
    use sqlx::PgPool;

    use crate::{
        Context,
        political_groups::{self, AuthorisedAgentId, PoliticalGroupId},
        test_utils::{response_body_string, sample_authorised_agent, sample_political_group},
    };

    #[sqlx::test]
    async fn list_authorised_agents_shows_created_agent(pool: PgPool) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_authorised_agent(&pool, political_group.id, &authorised_agent)
            .await?;

        let response = list_authorised_agents(
            AuthorisedAgentsPath {},
            Context::new_test(pool.clone()).await,
            political_group,
            State(pool.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&authorised_agent.last_name));

        Ok(())
    }

    #[sqlx::test]
    async fn list_authorised_agents_shows_edit_link(pool: PgPool) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_authorised_agent(&pool, political_group.id, &authorised_agent)
            .await?;

        let response = list_authorised_agents(
            AuthorisedAgentsPath {},
            Context::new_test(pool.clone()).await,
            political_group,
            State(pool.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&authorised_agent.edit_path()));

        Ok(())
    }
}
