use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, AppResponse, AppStore, Context, Form, HtmlTemplate,
    authorised_agents::AuthorisedAgent,
    common::Problematic,
    filters,
    form::{EmptyForm, FormData},
    redirect_success,
};

use super::AuthorisedAgentDeletePath;

#[derive(Template)]
#[template(path = "authorised_agents/pages/delete.html")]
struct DeleteAuthorisedAgentTemplate {
    authorised_agent: AuthorisedAgent,
    form: FormData<EmptyForm>,
}

pub async fn delete_authorised_agent_confirm(
    _: AuthorisedAgentDeletePath,
    context: Context,
    authorised_agent: AuthorisedAgent,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        DeleteAuthorisedAgentTemplate {
            form: FormData::new(&context.session.csrf_token),
            authorised_agent,
        },
        context,
    ))
}

pub async fn delete_authorised_agent(
    _: AuthorisedAgentDeletePath,
    authorised_agent: AuthorisedAgent,
    context: Context,
    store: AppStore,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.session.csrf_token) {
        Err(_) => Err(AppError::CsrfTokenInvalid),
        Ok(_) => {
            authorised_agent.delete(&store).await?;

            Ok(redirect_success(AuthorisedAgent::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum_extra::routing::TypedPath;

    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState, TokenValue,
        authorised_agents::AuthorisedAgentId,
        test_utils::{response_body_string, sample_authorised_agent},
    };

    #[tokio::test]
    async fn delete_authorised_agent_confirm_contains_delete_button() -> Result<(), AppError> {
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        let response = delete_authorised_agent_confirm(
            AuthorisedAgentDeletePath { agent_id },
            Context::new_test_without_db(),
            authorised_agent.clone(),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&authorised_agent.delete_path().to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn delete_authorised_agent_removes_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        authorised_agent.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();

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

        let agents = store.get_authorised_agents();
        assert!(agents.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn delete_authorised_agent_invalid_csrf_error_page() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);
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

        let agents = store.get_authorised_agents();
        assert_eq!(agents.len(), 1);

        Ok(())
    }
}
