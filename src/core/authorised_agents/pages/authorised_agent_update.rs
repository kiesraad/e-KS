use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentForm},
    filters,
    form::FormData,
};

use super::AuthorisedAgentUpdatePath;

#[derive(Template)]
#[template(path = "authorised_agents/update.html")]
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
            form: FormData::new_with_data(authorised_agent.clone().into(), &context.csrf_tokens),
            authorised_agent,
        },
        context,
    )
    .into_response())
}

pub async fn update_authorised_agent_submit(
    _: AuthorisedAgentUpdatePath,
    context: Context,
    authorised_agent: AuthorisedAgent,
    State(store): State<AppStore>,
    Form(form): Form<AuthorisedAgentForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&authorised_agent, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            AuthorisedAgentUpdateTemplate {
                authorised_agent,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(authorised_agent) => {
            authorised_agent.update(&store).await?;

            Ok(Redirect::to(&AuthorisedAgent::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, Form,
        authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
        political_groups::PoliticalGroupId,
        test_utils::{
            response_body_string, sample_authorised_agent, sample_authorised_agent_form,
            sample_political_group,
        },
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };

    #[tokio::test]
    async fn update_authorised_agent_renders_existing_agent() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
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
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_authorised_agent_form(&csrf_token);
        form.name.last_name = "Updated".to_string();

        let response = update_authorised_agent_submit(
            AuthorisedAgentUpdatePath { agent_id },
            context,
            authorised_agent.clone(),
            State(store.clone()),
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
        assert_eq!(location, AuthorisedAgent::list_path());

        let updated = store.get_authorised_agent(agent_id)?;
        assert_eq!(updated.name.last_name.to_string(), "Updated");

        Ok(())
    }

    #[tokio::test]
    async fn update_authorised_agent_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_group.create(&store).await?;
        authorised_agent.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_authorised_agent_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response = update_authorised_agent_submit(
            AuthorisedAgentUpdatePath { agent_id },
            context,
            authorised_agent.clone(),
            State(store),
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
