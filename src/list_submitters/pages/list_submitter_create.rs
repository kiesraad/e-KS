use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use super::ListSubmitterCreatePath;
use crate::{
    AppError, AppStore, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    list_submitters::{ListSubmitter, ListSubmitterForm},
};

#[derive(Template)]
#[template(path = "list_submitters/create.html")]
struct ListSubmitterCreateTemplate {
    form: FormData<ListSubmitterForm>,
}

pub async fn create_list_submitter(
    _: ListSubmitterCreatePath,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        ListSubmitterCreateTemplate {
            form: FormData::new(&context.csrf_tokens),
        },
        context,
    ))
}

pub async fn create_list_submitter_submit(
    _: ListSubmitterCreatePath,
    context: Context,
    State(store): State<AppStore>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            ListSubmitterCreateTemplate { form: form_data },
            context,
        )
        .into_response()),
        Ok(list_submitter) => {
            list_submitter.create(&store).await?;
            // TODO: set success flash message
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
        list_submitters::ListSubmitter,
        political_groups::PoliticalGroupId,
        test_utils::{response_body_string, sample_list_submitter_form, sample_political_group},
    };

    #[tokio::test]
    async fn create_list_submitter_renders_csrf_field() {
        let context = Context::new_test_without_db();

        let response = create_list_submitter(ListSubmitterCreatePath {}, context)
            .await
            .unwrap()
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains(&format!("action=\"{}\"", ListSubmitter::create_path())));
    }

    #[sqlx::test]
    async fn create_list_submitter_persists_and_redirects(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_list_submitter_form(&csrf_token);

        let response = create_list_submitter_submit(
            ListSubmitterCreatePath {},
            context,
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

        let submitters = store.get_list_submitters()?;
        assert_eq!(submitters.len(), 1);

        Ok(())
    }

    #[sqlx::test]
    async fn create_list_submitter_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_list_submitter_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = create_list_submitter_submit(
            ListSubmitterCreatePath {},
            context,
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
