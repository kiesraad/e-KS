use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, Context,
    form::{EmptyForm, Validate},
    political_groups::{self, PoliticalGroup},
};

use super::{AuthorisedAgentDeletePath, AuthorisedAgentEditPath};

pub async fn delete_authorised_agent(
    AuthorisedAgentDeletePath { agent_id }: AuthorisedAgentDeletePath,
    political_group: PoliticalGroup,
    context: Context,
    State(pool): State<PgPool>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => {
            Ok(Redirect::to(&AuthorisedAgentEditPath { agent_id }.to_string()).into_response())
        }
        Ok(_) => {
            political_groups::remove_authorised_agent(&pool, political_group.id, agent_id).await?;

            Ok(Redirect::to(&political_groups::AuthorisedAgent::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        Context, TokenValue,
        political_groups::{self, AuthorisedAgent, AuthorisedAgentId, PoliticalGroupId},
        test_utils::{sample_authorised_agent, sample_political_group},
    };

    #[sqlx::test]
    async fn delete_authorised_agent_removes_and_redirects(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_authorised_agent(&pool, political_group.id, &authorised_agent)
            .await?;
        political_groups::update_political_group(&pool, &political_group).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_authorised_agent(
            AuthorisedAgentDeletePath { agent_id },
            political_group,
            context,
            State(pool.clone()),
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
        assert_eq!(location, AuthorisedAgent::list_path());

        let agents = political_groups::get_authorised_agents(&pool, group_id).await?;
        assert!(agents.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn delete_authorised_agent_invalid_csrf_redirects_to_edit(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_authorised_agent(&pool, political_group.id, &authorised_agent)
            .await?;
        political_groups::update_political_group(&pool, &political_group).await?;

        let context = Context::new_test(pool.clone()).await;

        let response = delete_authorised_agent(
            AuthorisedAgentDeletePath { agent_id },
            political_group,
            context,
            State(pool.clone()),
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
        assert_eq!(location, AuthorisedAgentEditPath { agent_id }.to_string());

        let agents = political_groups::get_authorised_agents(&pool, group_id).await?;
        assert_eq!(agents.len(), 1);

        Ok(())
    }
}
