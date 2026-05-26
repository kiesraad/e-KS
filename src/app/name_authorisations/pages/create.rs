use askama::Template;
use axum::response::{IntoResponse, Response};

use super::NameAuthorisationCreatePath;
use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate, filters,
    form::FormData,
    name_authorisations::{NameAuthorisation, NameAuthorisationForm},
    redirect_success,
};

#[derive(Template)]
#[template(path = "name_authorisations/pages/create.html")]
struct NameAuthorisationCreateTemplate {
    form: FormData<NameAuthorisationForm>,
}

pub async fn create_name_authorisation(
    _: NameAuthorisationCreatePath,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        NameAuthorisationCreateTemplate {
            form: FormData::new(&context.session.csrf_token),
        },
        context,
    ))
}

pub async fn create_name_authorisation_submit(
    _: NameAuthorisationCreatePath,
    context: Context,
    store: AppStore,
    Form(form): Form<NameAuthorisationForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.session.csrf_token) {
        Err(form_data) => Ok(HtmlTemplate(
            NameAuthorisationCreateTemplate { form: form_data },
            context,
        )
        .into_response()),
        Ok(authorisation) => {
            authorisation.create(&store).await?;

            Ok(redirect_success(NameAuthorisation::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState,
        test_utils::{response_body_string, sample_name_authorisation_form},
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::routing::TypedPath;

    #[tokio::test]
    async fn create_name_authorisation_renders_csrf_field() -> Result<(), AppError> {
        let response = create_name_authorisation(
            NameAuthorisationCreatePath {},
            Context::new_test_without_db(),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));

        Ok(())
    }

    #[tokio::test]
    async fn create_name_authorisation_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();
        let form = sample_name_authorisation_form(&csrf_token);

        let response = create_name_authorisation_submit(
            NameAuthorisationCreatePath {},
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
        let authorisations = store.get_name_authorisations();
        assert_eq!(authorisations.len(), 1);
        assert_eq!(
            location,
            NameAuthorisation::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        Ok(())
    }

    #[tokio::test]
    async fn create_name_authorisation_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();
        let mut form = sample_name_authorisation_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response = create_name_authorisation_submit(
            NameAuthorisationCreatePath {},
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
