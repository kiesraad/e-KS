use askama::Template;
use axum::response::{IntoResponse, Response};

use super::AuthorisedAgentCreatePath;
use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentForm},
    filters,
    form::FormData,
    redirect_success,
};

#[derive(Template)]
#[template(path = "authorised_agents/pages/create.html")]
struct AuthorisedAgentCreateTemplate {
    form: FormData<AuthorisedAgentForm>,
}

pub async fn create_authorised_agent(
    _: AuthorisedAgentCreatePath,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        AuthorisedAgentCreateTemplate {
            form: FormData::new(&context.session.csrf_tokens),
        },
        context,
    ))
}

pub async fn create_authorised_agent_submit(
    _: AuthorisedAgentCreatePath,
    context: Context,
    store: AppStore,
    Form(form): Form<AuthorisedAgentForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.session.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            AuthorisedAgentCreateTemplate { form: form_data },
            context,
        )
        .into_response()),
        Ok(authorised_agent) => {
            authorised_agent.create(&store).await?;

            Ok(redirect_success(AuthorisedAgent::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState,
        test_utils::{response_body_string, sample_authorised_agent_form},
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::routing::TypedPath;

    #[tokio::test]
    async fn create_authorised_agent_renders_csrf_field() -> Result<(), AppError> {
        let response =
            create_authorised_agent(AuthorisedAgentCreatePath {}, Context::new_test_without_db())
                .await
                .unwrap()
                .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));

        Ok(())
    }

    #[tokio::test]
    async fn create_authorised_agent_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let form = sample_authorised_agent_form(&csrf_token);

        let response = create_authorised_agent_submit(
            AuthorisedAgentCreatePath {},
            context,
            store.clone(),
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
        let agents = store.get_authorised_agents();
        assert_eq!(agents.len(), 1);
        assert_eq!(
            location,
            AuthorisedAgent::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        Ok(())
    }

    #[tokio::test]
    async fn create_authorised_agent_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let mut form = sample_authorised_agent_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response = create_authorised_agent_submit(
            AuthorisedAgentCreatePath {},
            context,
            store,
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
