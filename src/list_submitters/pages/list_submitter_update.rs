use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    list_submitters::{ListSubmitter, ListSubmitterForm},
};

use super::ListSubmitterUpdatePath;

#[derive(Template)]
#[template(path = "list_submitters/update.html")]
struct ListSubmitterUpdateTemplate {
    list_submitter: ListSubmitter,
    form: FormData<ListSubmitterForm>,
}

pub async fn update_list_submitter(
    _: ListSubmitterUpdatePath,
    context: Context,
    list_submitter: ListSubmitter,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(
        ListSubmitterUpdateTemplate {
            form: FormData::new_with_data(list_submitter.clone().into(), &context.csrf_tokens),
            list_submitter,
        },
        context,
    )
    .into_response())
}

pub async fn update_list_submitter_submit(
    _: ListSubmitterUpdatePath,
    context: Context,
    list_submitter: ListSubmitter,
    State(store): State<AppStore>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&list_submitter, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            ListSubmitterUpdateTemplate {
                list_submitter,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(list_submitter) => {
            list_submitter.update(&store).await?;

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
        AppError, AppStore, Context,
        list_submitters::{ListSubmitter, ListSubmitterId},
        political_groups::PoliticalGroupId,
        test_utils::{
            response_body_string, sample_list_submitter, sample_list_submitter_form,
            sample_political_group,
        },
    };

    #[sqlx::test]
    async fn update_list_submitter_renders_existing_submitter(
        pool: PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_group.create(&store).await?;
        list_submitter.create(&store).await?;

        let response = update_list_submitter(
            ListSubmitterUpdatePath { submitter_id },
            Context::new_test_without_db(),
            list_submitter.clone(),
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
    async fn update_list_submitter_persists_and_redirects(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_group.create(&store).await?;
        list_submitter.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_list_submitter_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update_list_submitter_submit(
            ListSubmitterUpdatePath { submitter_id },
            context,
            list_submitter.clone(),
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
        assert_eq!(location, ListSubmitter::list_path());

        let updated = store.get_list_submitter(submitter_id)?;
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn update_list_submitter_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_group.create(&store).await?;
        list_submitter.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_list_submitter_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update_list_submitter_submit(
            ListSubmitterUpdatePath { submitter_id },
            context,
            list_submitter.clone(),
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
