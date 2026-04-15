use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, Context, Form, HtmlTemplate, RequestCtx,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentForm},
    filters,
    form::FormData,
    redirect_success,
};

use super::AuthorisedAgentUpdatePath;

#[derive(Template)]
#[template(path = "authorised_agents/pages/update.html")]
struct AuthorisedAgentUpdateTemplate {
    authorised_agent: AuthorisedAgent,
    form: FormData<AuthorisedAgentForm>,
}

pub async fn update_authorised_agent(
    _: AuthorisedAgentUpdatePath,
    context: Context,
    authorised_agent: AuthorisedAgent,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(
        AuthorisedAgentUpdateTemplate {
            form: FormData::new_with_data(
                authorised_agent.clone().into(),
                &context.session.csrf_tokens,
            ),
            authorised_agent,
        },
        context,
    )
    .into_response())
}

pub async fn update_authorised_agent_submit(
    _: AuthorisedAgentUpdatePath,
    ctx: RequestCtx,
    authorised_agent: AuthorisedAgent,
    Form(form): Form<AuthorisedAgentForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&authorised_agent, ctx.csrf()) {
        Err(form_data) => Ok(HtmlTemplate(
            AuthorisedAgentUpdateTemplate {
                authorised_agent,
                form: form_data,
            },
            ctx.context,
        )
        .into_response()),
        Ok(authorised_agent) => {
            authorised_agent.update(&ctx.store).await?;

            Ok(redirect_success(AuthorisedAgent::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState,
        authorised_agents::AuthorisedAgentId,
        test_utils::{response_body_string, sample_authorised_agent, sample_authorised_agent_form},
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::routing::TypedPath;

    #[tokio::test]
    async fn update_authorised_agent_renders_existing_agent() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        authorised_agent.create(&store).await?;

        let response = update_authorised_agent(
            AuthorisedAgentUpdatePath { agent_id },
            Context::new_test_without_db(),
            authorised_agent.clone(),
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
    async fn update_authorised_agent_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);
        authorised_agent.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let mut form = sample_authorised_agent_form(&csrf_token);
        form.name.last_name = "Updated".to_string();

        let response = update_authorised_agent_submit(
            AuthorisedAgentUpdatePath { agent_id },
            RequestCtx {
                context,
                store: store.clone(),
                query: QueryParamState::default(),
            },
            authorised_agent.clone(),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(
            location,
            AuthorisedAgent::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let updated = store.get_authorised_agent(agent_id)?;
        assert_eq!(updated.name.last_name.to_string(), "Updated");

        Ok(())
    }

    #[tokio::test]
    async fn update_authorised_agent_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);
        authorised_agent.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let mut form = sample_authorised_agent_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response = update_authorised_agent_submit(
            AuthorisedAgentUpdatePath { agent_id },
            RequestCtx {
                context,
                store,
                query: QueryParamState::default(),
            },
            authorised_agent.clone(),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("This field must not be empty."));

        Ok(())
    }
}
