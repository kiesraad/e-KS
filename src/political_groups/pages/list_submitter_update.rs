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
        self, AuthorisedAgent, ListSubmitter, ListSubmitterForm, PoliticalGroup,
        SubstituteSubmitter,
    },
};

use super::ListSubmitterEditPath;

#[derive(Template)]
#[template(path = "political_groups/list_submitter_update.html")]
struct ListSubmitterUpdateTemplate {
    list_submitters: Vec<ListSubmitter>,
    list_submitter: ListSubmitter,
    form: FormData<ListSubmitterForm>,
}

pub async fn edit_list_submitter(
    ListSubmitterEditPath { submitter_id }: ListSubmitterEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
) -> Result<Response, AppError> {
    let list_submitter =
        political_groups::get_list_submitter(&pool, political_group.id, &submitter_id).await?;
    let list_submitters = political_groups::get_list_submitters(&pool, political_group.id).await?;

    Ok(HtmlTemplate(
        ListSubmitterUpdateTemplate {
            form: FormData::new_with_data(list_submitter.clone().into(), &context.csrf_tokens),
            list_submitter,
            list_submitters,
        },
        context,
    )
    .into_response())
}

pub async fn update_list_submitter(
    ListSubmitterEditPath { submitter_id }: ListSubmitterEditPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    let list_submitter =
        political_groups::get_list_submitter(&pool, political_group.id, &submitter_id).await?;
    let list_submitters = political_groups::get_list_submitters(&pool, political_group.id).await?;

    match form.validate_update(&list_submitter, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            ListSubmitterUpdateTemplate {
                list_submitter,
                form: form_data,
                list_submitters,
            },
            context,
        )
        .into_response()),
        Ok(list_submitter) => {
            political_groups::update_list_submitter(&pool, political_group.id, &list_submitter)
                .await?;

            Ok(Redirect::to(&ListSubmitter::list_path()).into_response())
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
        political_groups::{self, ListSubmitter, ListSubmitterId, PoliticalGroupId},
        test_utils::{
            response_body_string, sample_list_submitter, sample_list_submitter_form,
            sample_political_group,
        },
    };

    #[sqlx::test]
    async fn edit_list_submitter_renders_existing_submitter(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_list_submitter(&pool, political_group.id, &list_submitter).await?;

        let response = edit_list_submitter(
            ListSubmitterEditPath { submitter_id },
            Context::new_test(pool.clone()).await,
            political_group,
            State(pool.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list_submitter.last_name));

        Ok(())
    }

    #[sqlx::test]
    async fn update_list_submitter_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_list_submitter(&pool, political_group.id, &list_submitter).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_list_submitter_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update_list_submitter(
            ListSubmitterEditPath { submitter_id },
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
        assert_eq!(location, ListSubmitter::list_path());

        let updated = political_groups::get_list_submitter(&pool, group_id, &submitter_id).await?;
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn update_list_submitter_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_list_submitter(&pool, political_group.id, &list_submitter).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_list_submitter_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update_list_submitter(
            ListSubmitterEditPath { submitter_id },
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
