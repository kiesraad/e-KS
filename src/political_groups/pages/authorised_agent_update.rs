use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    political_groups::{
        self, AuthorisedAgent, AuthorisedAgentForm, ListSubmitter, PoliticalGroup,
        SubstituteSubmitter,
    },
};

use super::AuthorisedAgentEditPath;

#[derive(Template)]
#[template(path = "political_groups/authorised_agent_update.html")]
struct AuthorisedAgentUpdateTemplate {
    authorised_agents: Vec<AuthorisedAgent>,
    authorised_agent: AuthorisedAgent,
    form: FormData<AuthorisedAgentForm>,
}

pub async fn edit_authorised_agent(
    AuthorisedAgentEditPath { agent_id }: AuthorisedAgentEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
) -> Result<Response, AppError> {
    let authorised_agent =
        political_groups::get_authorised_agent(&pool, political_group.id, &agent_id).await?;
    let authorised_agents =
        political_groups::get_authorised_agents(&pool, political_group.id).await?;

    Ok(HtmlTemplate(
        AuthorisedAgentUpdateTemplate {
            form: FormData::new_with_data(authorised_agent.clone().into(), &context.csrf_tokens),
            authorised_agent,
            authorised_agents,
        },
        context,
    )
    .into_response())
}

pub async fn update_authorised_agent(
    AuthorisedAgentEditPath { agent_id }: AuthorisedAgentEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
    Form(form): Form<AuthorisedAgentForm>,
) -> Result<Response, AppError> {
    let authorised_agent =
        political_groups::get_authorised_agent(&pool, political_group.id, &agent_id).await?;
    let authorised_agents =
        political_groups::get_authorised_agents(&pool, political_group.id).await?;

    match form.validate_update(&authorised_agent, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            AuthorisedAgentUpdateTemplate {
                authorised_agent,
                form: form_data,
                authorised_agents,
            },
            context,
        )
        .into_response()),
        Ok(authorised_agent) => {
            political_groups::update_authorised_agent(&pool, political_group.id, &authorised_agent)
                .await?;

            Ok(Redirect::to(&AuthorisedAgent::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        Context,
        political_groups::{self, AuthorisedAgent, AuthorisedAgentId, PoliticalGroupId},
        test_utils::{
            response_body_string, sample_authorised_agent, sample_authorised_agent_form,
            sample_political_group,
        },
    };

    #[sqlx::test]
    async fn edit_authorised_agent_renders_existing_agent(pool: PgPool) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_authorised_agent(&pool, political_group.id, &authorised_agent)
            .await?;

        let response = edit_authorised_agent(
            AuthorisedAgentEditPath { agent_id },
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
    async fn update_authorised_agent_persists_and_redirects(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_authorised_agent(&pool, political_group.id, &authorised_agent)
            .await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_authorised_agent_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update_authorised_agent(
            AuthorisedAgentEditPath { agent_id },
            context,
            political_group,
            State(pool.clone()),
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

        let updated = political_groups::get_authorised_agent(&pool, group_id, &agent_id).await?;
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn update_authorised_agent_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let agent_id = AuthorisedAgentId::new();
        let authorised_agent = sample_authorised_agent(agent_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_authorised_agent(&pool, political_group.id, &authorised_agent)
            .await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_authorised_agent_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update_authorised_agent(
            AuthorisedAgentEditPath { agent_id },
            context,
            political_group,
            State(pool.clone()),
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
