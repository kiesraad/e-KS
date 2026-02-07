use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{AppError, AppStore, Context, authorised_agents::AuthorisedAgent, form::EmptyForm};

use super::AuthorisedAgentDeletePath;

pub async fn delete_authorised_agent(
    _: AuthorisedAgentDeletePath,
    authorized_agent: AuthorisedAgent,
    context: Context,
    State(store): State<AppStore>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => Ok(Redirect::to(&authorized_agent.update_path()).into_response()),
        Ok(_) => {
            authorized_agent.delete(&store).await?;

            Ok(Redirect::to(&AuthorisedAgent::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, TokenValue,
        authorised_agents::AuthorisedAgentId,
        political_groups::PoliticalGroupId,
        test_utils::{sample_authorised_agent, sample_political_group},
    };
    use axum_extra::extract::Form;

    #[tokio::test]
    async fn delete_authorised_agent_removes_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_authorised_agent(
            AuthorisedAgentDeletePath { agent_id },
            authorised_agent,
            context,
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
        assert_eq!(location, AuthorisedAgent::list_path());

        let agents = store.get_authorised_agents()?;
        assert!(agents.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn delete_authorised_agent_invalid_csrf_redirects_to_edit() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store).await?;

        let context = Context::new_test_without_db();

        let response = delete_authorised_agent(
            AuthorisedAgentDeletePath { agent_id },
            authorised_agent.clone(),
            context,
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
        assert_eq!(location, authorised_agent.update_path());

        let agents = store.get_authorised_agents()?;
        assert_eq!(agents.len(), 1);

        Ok(())
    }
}
