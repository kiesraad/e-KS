use super::ListSubmittersPath;
use crate::{
    AppError, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    political_groups::{self, ListSubmitter, PoliticalGroup, PreferredSubmitterForm},
};
use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "political_groups/list_submitters.html")]
struct ListSubmittersTemplate {
    list_submitters: Vec<ListSubmitter>,
    form: FormData<PreferredSubmitterForm>,
}

pub async fn list_submitters(
    _: ListSubmittersPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AppError> {
    let list_submitters = political_groups::get_list_submitters(&pool, political_group.id).await?;

    Ok(HtmlTemplate(
        ListSubmittersTemplate {
            list_submitters,
            form: FormData::new_with_data(political_group.clone().into(), &context.csrf_tokens),
        },
        context,
    ))
}

pub async fn update_list_submitters(
    _: ListSubmittersPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
    Form(form): Form<PreferredSubmitterForm>,
) -> Result<Response, AppError> {
    let list_submitters = political_groups::get_list_submitters(&pool, political_group.id).await?;

    match form.validate_update(political_group.clone(), &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            ListSubmittersTemplate {
                form: form_data,
                list_submitters,
            },
            context,
        )
        .into_response()),
        Ok(form_data) => {
            political_groups::set_default_list_submitter(
                &pool,
                political_group.id,
                form_data.list_submitter_id,
            )
            .await?;

            Ok(Redirect::to(&ListSubmitter::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};
    use sqlx::PgPool;

    use crate::{
        Context,
        political_groups::{self, ListSubmitterId, PoliticalGroupId},
        test_utils::{response_body_string, sample_list_submitter, sample_political_group},
    };

    #[sqlx::test]
    async fn list_submitters_shows_created_submitter(pool: PgPool) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_list_submitter(&pool, political_group.id, &list_submitter).await?;

        let response = list_submitters(
            ListSubmittersPath {},
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
    async fn list_submitters_shows_edit_link(pool: PgPool) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_list_submitter(&pool, political_group.id, &list_submitter).await?;

        let response = list_submitters(
            ListSubmittersPath {},
            Context::new_test(pool.clone()).await,
            political_group,
            State(pool.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list_submitter.edit_path()));

        Ok(())
    }
}
