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
        self, AuthorisedAgent, ListSubmitter, PoliticalGroup, PoliticalGroupForm,
        SubstituteSubmitter,
    },
};

use super::PoliticalGroupEditPath;

#[derive(Template)]
#[template(path = "political_groups/update.html")]
struct PoliticalGroupUpdateTemplate {
    form: FormData<PoliticalGroupForm>,
}

pub async fn edit_political_group(
    _: PoliticalGroupEditPath,
    context: Context,
    political_group: PoliticalGroup,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(
        PoliticalGroupUpdateTemplate {
            form: FormData::new_with_data(political_group.clone().into(), &context.csrf_tokens),
        },
        context,
    )
    .into_response())
}

pub async fn update_political_group(
    _: PoliticalGroupEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
    Form(form): Form<PoliticalGroupForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&political_group, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PoliticalGroupUpdateTemplate { form: form_data },
            context,
        )
        .into_response()),
        Ok(political_group) => {
            political_groups::update_political_group(&pool, &political_group).await?;

            Ok(Redirect::to(&PoliticalGroup::edit_path()).into_response())
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
        political_groups::{self, AuthorisedAgentId, PoliticalGroupId},
        test_utils::{
            response_body_string, sample_authorised_agent, sample_political_group,
            sample_political_group_form,
        },
    };

    #[sqlx::test]
    async fn edit_political_group_renders_existing_data(pool: PgPool) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);

        political_groups::create_political_group(&pool, &political_group).await?;

        let response = edit_political_group(
            PoliticalGroupEditPath {},
            Context::new_test(pool.clone()).await,
            political_group,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains("Kiesraad Demo"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_political_group_persists_and_redirects(
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
        let form = sample_political_group_form(&csrf_token);

        let response = update_political_group(
            PoliticalGroupEditPath {},
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
        assert_eq!(location, PoliticalGroup::edit_path());

        let updated = political_groups::get_single_political_group(&pool)
            .await?
            .expect("political group");
        assert_eq!(updated.long_list_allowed, Some(true));
        assert_eq!(updated.legal_name, Some("Updated Legal Name".to_string()));
        assert_eq!(
            updated.display_name,
            Some("Updated Display Name".to_string())
        );

        Ok(())
    }

    #[sqlx::test]
    async fn update_political_group_invalid_form_renders_template(
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
        let mut form = sample_political_group_form(&csrf_token);

        form.display_name = "!".to_string(); // Invalid value

        let response = update_political_group(
            PoliticalGroupEditPath {},
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
        assert!(body.contains("The value is too short"));

        Ok(())
    }
}
