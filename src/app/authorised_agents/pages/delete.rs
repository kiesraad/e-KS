use crate::{
    AppError, AppStore, Context, Form, authorised_agents::AuthorisedAgent, form::EmptyForm,
    redirect_success,
};
use axum::response::Response;

use super::AuthorisedAgentDeletePath;

pub async fn delete_authorised_agent(
    _: AuthorisedAgentDeletePath,
    authorized_agent: AuthorisedAgent,
    context: Context,
    store: AppStore,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.session.csrf_tokens) {
        Err(_) => Err(AppError::CsrfTokenInvalid),
        Ok(_) => {
            authorized_agent.delete(&store).await?;

            Ok(redirect_success(AuthorisedAgent::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum_extra::routing::TypedPath;

    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, PoliticalGroupId, QueryParamState, TokenValue,
        authorised_agents::AuthorisedAgentId,
        test_utils::{sample_authorised_agent, sample_political_group},
    };

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
        let csrf_token = context.session.csrf_tokens.issue().value;

        let response = delete_authorised_agent(
            AuthorisedAgentDeletePath { agent_id },
            authorised_agent,
            context,
            store.clone(),
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
        assert_eq!(
            location,
            AuthorisedAgent::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let agents = store.get_authorised_agents()?;
        assert!(agents.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn delete_authorised_agent_invalid_csrf_error_page() -> Result<(), AppError> {
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
            store.clone(),
            Form(EmptyForm::new(TokenValue("invalid".to_string()))),
        )
        .await
        .unwrap_err();

        assert!(matches!(response, AppError::CsrfTokenInvalid));

        let agents = store.get_authorised_agents()?;
        assert_eq!(agents.len(), 1);

        Ok(())
    }
}
