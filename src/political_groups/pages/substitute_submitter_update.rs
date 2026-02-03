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
        self, AuthorisedAgent, ListSubmitter, PoliticalGroup, SubstituteSubmitter,
        SubstituteSubmitterForm,
    },
};

use super::SubstituteSubmitterEditPath;

#[derive(Template)]
#[template(path = "political_groups/substitute_submitter_update.html")]
struct SubstituteSubmitterUpdateTemplate {
    substitute_submitters: Vec<SubstituteSubmitter>,
    substitute_submitter: SubstituteSubmitter,
    form: FormData<SubstituteSubmitterForm>,
}

pub async fn edit_substitute_submitter(
    SubstituteSubmitterEditPath { submitter_id }: SubstituteSubmitterEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
) -> Result<Response, AppError> {
    let substitute_submitter =
        political_groups::get_substitute_submitter(&pool, political_group.id, &submitter_id)
            .await?;
    let substitute_submitters =
        political_groups::get_substitute_submitters(&pool, political_group.id).await?;

    Ok(HtmlTemplate(
        SubstituteSubmitterUpdateTemplate {
            form: FormData::new_with_data(
                substitute_submitter.clone().into(),
                &context.csrf_tokens,
            ),
            substitute_submitter,
            substitute_submitters,
        },
        context,
    )
    .into_response())
}

pub async fn update_substitute_submitter(
    SubstituteSubmitterEditPath { submitter_id }: SubstituteSubmitterEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
    Form(form): Form<SubstituteSubmitterForm>,
) -> Result<Response, AppError> {
    let substitute_submitter =
        political_groups::get_substitute_submitter(&pool, political_group.id, &submitter_id)
            .await?;
    let substitute_submitters =
        political_groups::get_substitute_submitters(&pool, political_group.id).await?;

    match form.validate_update(&substitute_submitter, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            SubstituteSubmitterUpdateTemplate {
                substitute_submitter,
                form: form_data,
                substitute_submitters,
            },
            context,
        )
        .into_response()),
        Ok(substitute_submitter) => {
            political_groups::update_substitute_submitter(
                &pool,
                political_group.id,
                &substitute_submitter,
            )
            .await?;

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
        political_groups::{self, PoliticalGroupId, SubstituteSubmitter, SubstituteSubmitterId},
        test_utils::{
            response_body_string, sample_political_group, sample_substitute_submitter,
            sample_substitute_submitter_form,
        },
    };

    #[sqlx::test]
    async fn edit_substitute_submitter_renders_existing_submitter(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_substitute_submitter(
            &pool,
            political_group.id,
            &substitute_submitter,
        )
        .await?;

        let response = edit_substitute_submitter(
            SubstituteSubmitterEditPath { submitter_id },
            Context::new_test(pool.clone()).await,
            political_group,
            State(pool.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&substitute_submitter.last_name));

        Ok(())
    }

    #[sqlx::test]
    async fn update_substitute_submitter_persists_and_redirects(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_substitute_submitter(
            &pool,
            political_group.id,
            &substitute_submitter,
        )
        .await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_substitute_submitter_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update_substitute_submitter(
            SubstituteSubmitterEditPath { submitter_id },
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

        let updated =
            political_groups::get_substitute_submitter(&pool, group_id, &submitter_id).await?;
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn update_substitute_submitter_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_substitute_submitter(
            &pool,
            political_group.id,
            &substitute_submitter,
        )
        .await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_substitute_submitter_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update_substitute_submitter(
            SubstituteSubmitterEditPath { submitter_id },
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
