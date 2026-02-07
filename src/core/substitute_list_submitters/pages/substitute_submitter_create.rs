use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use super::SubstituteSubmitterCreatePath;
use crate::{
    AppError, AppStore, Context, HtmlTemplate, filters,
    form::FormData,
    list_submitters::ListSubmitter,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterForm},
};

#[derive(Template)]
#[template(path = "substitute_list_submitters/create.html")]
struct SubstituteSubmitterCreateTemplate {
    form: FormData<SubstituteSubmitterForm>,
}

pub async fn create_substitute_submitter(
    _: SubstituteSubmitterCreatePath,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        SubstituteSubmitterCreateTemplate {
            form: FormData::new(&context.csrf_tokens),
        },
        context,
    ))
}

pub async fn create_substitute_submitter_submit(
    _: SubstituteSubmitterCreatePath,
    context: Context,
    State(store): State<AppStore>,
    Form(form): Form<SubstituteSubmitterForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            SubstituteSubmitterCreateTemplate { form: form_data },
            context,
        )
        .into_response()),
        Ok(substitute_submitter) => {
            substitute_submitter.create(&store).await?;
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

    use crate::{
        AppError, AppStore, Context,
        political_groups::PoliticalGroupId,
        substitute_list_submitters::SubstituteSubmitter,
        test_utils::{
            response_body_string, sample_political_group, sample_substitute_submitter_form,
        },
    };

    #[tokio::test]
    async fn create_substitute_submitter_renders_csrf_field() {
        let context = Context::new_test_without_db();

        let response = create_substitute_submitter(SubstituteSubmitterCreatePath {}, context)
            .await
            .unwrap()
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains(&format!(
            "action=\"{}\"",
            SubstituteSubmitter::create_path()
        )));
    }

    #[tokio::test]
    async fn create_substitute_submitter_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_substitute_submitter_form(&csrf_token);

        let response = create_substitute_submitter_submit(
            SubstituteSubmitterCreatePath {},
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

        let submitters = store.get_substitute_submitters()?;
        assert_eq!(submitters.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn create_substitute_submitter_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_substitute_submitter_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response = create_substitute_submitter_submit(
            SubstituteSubmitterCreatePath {},
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
