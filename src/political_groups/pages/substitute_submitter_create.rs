use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use super::SubstituteSubmitterNewPath;
use crate::{
    AppError, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    political_groups::{
        self, AuthorisedAgent, ListSubmitter, PoliticalGroup, SubstituteSubmitter,
        SubstituteSubmitterForm,
    },
};

#[derive(Template)]
#[template(path = "political_groups/substitute_submitter_create.html")]
struct SubstituteSubmitterCreateTemplate {
    substitute_submitters: Vec<SubstituteSubmitter>,
    form: FormData<SubstituteSubmitterForm>,
}

pub async fn new_substitute_submitter_form(
    _: SubstituteSubmitterNewPath,
    context: Context,
    State(pool): State<PgPool>,
    political_group: PoliticalGroup,
) -> Result<impl IntoResponse, AppError> {
    let substitute_submitters =
        political_groups::get_substitute_submitters(&pool, political_group.id).await?;

    Ok(HtmlTemplate(
        SubstituteSubmitterCreateTemplate {
            substitute_submitters,
            form: FormData::new(&context.csrf_tokens),
        },
        context,
    ))
}

pub async fn create_substitute_submitter(
    _: SubstituteSubmitterNewPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
    Form(form): Form<SubstituteSubmitterForm>,
) -> Result<Response, AppError> {
    let substitute_submitters =
        political_groups::get_substitute_submitters(&pool, political_group.id).await?;

    match form.validate_create(&context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            SubstituteSubmitterCreateTemplate {
                substitute_submitters,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(substitute_submitter) => {
            political_groups::create_substitute_submitter(
                &pool,
                political_group.id,
                &substitute_submitter,
            )
            .await?;
            // TODO: set success flash message
            Ok(Redirect::to(&SubstituteSubmitter::list_path()).into_response())
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
        political_groups::{self, PoliticalGroupId, SubstituteSubmitter},
        test_utils::{
            response_body_string, sample_political_group, sample_substitute_submitter_form,
        },
    };

    #[sqlx::test]
    async fn new_substitute_submitter_form_renders_csrf_field(pool: PgPool) {
        let context = Context::new_test(pool.clone()).await;
        let group_id = PoliticalGroupId::new();

        let response = new_substitute_submitter_form(
            SubstituteSubmitterNewPath {},
            context,
            State(pool),
            sample_political_group(group_id),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains(&format!("action=\"{}\"", SubstituteSubmitter::new_path())));
    }

    #[sqlx::test]
    async fn create_substitute_submitter_persists_and_redirects(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_groups::create_political_group(&pool, &political_group).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_substitute_submitter_form(&csrf_token);

        let response = create_substitute_submitter(
            SubstituteSubmitterNewPath {},
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
        assert_eq!(location, SubstituteSubmitter::list_path());

        let submitters = political_groups::get_substitute_submitters(&pool, group_id).await?;
        assert_eq!(submitters.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn create_substitute_submitter_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_groups::create_political_group(&pool, &political_group).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_substitute_submitter_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = create_substitute_submitter(
            SubstituteSubmitterNewPath {},
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
