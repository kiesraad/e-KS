use askama::Template;
use axum::response::{IntoResponse, Response};

use super::SubstituteSubmitterCreatePath;
use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate, filters,
    form::FormData,
    list_submitters::ListSubmitter,
    redirect_success,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterForm},
};

#[derive(Template)]
#[template(path = "substitute_list_submitters/pages/create.html")]
struct SubstituteSubmitterCreateTemplate {
    form: FormData<SubstituteSubmitterForm>,
}

pub async fn create_substitute_submitter(
    _: SubstituteSubmitterCreatePath,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        SubstituteSubmitterCreateTemplate {
            form: FormData::new(&context.session.csrf_tokens),
        },
        context,
    ))
}

pub async fn create_substitute_submitter_submit(
    _: SubstituteSubmitterCreatePath,
    context: Context,
    store: AppStore,
    Form(form): Form<SubstituteSubmitterForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.session.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            SubstituteSubmitterCreateTemplate { form: form_data },
            context,
        )
        .into_response()),
        Ok(substitute_submitter) => {
            substitute_submitter.create(&store).await?;

            Ok(redirect_success(ListSubmitter::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QueryParamState;
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::routing::TypedPath;

    use crate::{
        AppError, AppStore, Context, PoliticalGroupId,
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
    }

    #[tokio::test]
    async fn create_substitute_submitter_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let form = sample_substitute_submitter_form(&csrf_token);

        let response = create_substitute_submitter_submit(
            SubstituteSubmitterCreatePath {},
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
        let submitters = store.get_substitute_submitters()?;
        assert_eq!(submitters.len(), 1);
        assert_eq!(
            location,
            ListSubmitter::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        Ok(())
    }

    #[tokio::test]
    async fn create_substitute_submitter_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        political_group.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let mut form = sample_substitute_submitter_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response = create_substitute_submitter_submit(
            SubstituteSubmitterCreatePath {},
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
