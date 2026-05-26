use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, AppResponse, AppStore, Context, Form, HtmlTemplate,
    common::Problematic,
    filters,
    form::{EmptyForm, FormData},
    name_authorisations::NameAuthorisation,
    redirect_success,
};

use super::NameAuthorisationDeletePath;

#[derive(Template)]
#[template(path = "name_authorisations/pages/delete.html")]
struct DeleteNameAuthorisationTemplate {
    name_authorisation: NameAuthorisation,
    form: FormData<EmptyForm>,
}

pub async fn delete_name_authorisation_confirm(
    _: NameAuthorisationDeletePath,
    context: Context,
    name_authorisation: NameAuthorisation,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        DeleteNameAuthorisationTemplate {
            form: FormData::new(&context.session.csrf_token),
            name_authorisation,
        },
        context,
    ))
}

pub async fn delete_name_authorisation(
    _: NameAuthorisationDeletePath,
    name_authorisation: NameAuthorisation,
    context: Context,
    store: AppStore,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.session.csrf_token) {
        Err(_) => Err(AppError::CsrfTokenInvalid),
        Ok(_) => {
            name_authorisation.delete(&store).await?;

            Ok(redirect_success(NameAuthorisation::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum_extra::routing::TypedPath;

    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState, TokenValue,
        name_authorisations::NameAuthorisationId,
        test_utils::{response_body_string, sample_name_authorisation},
    };

    #[tokio::test]
    async fn delete_name_authorisation_confirm_contains_delete_button() -> Result<(), AppError> {
        let authorisation_id = NameAuthorisationId::new();
        let name_authorisation = sample_name_authorisation(authorisation_id);

        let response = delete_name_authorisation_confirm(
            NameAuthorisationDeletePath { authorisation_id },
            Context::new_test_without_db(),
            name_authorisation.clone(),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&name_authorisation.delete_path().to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn delete_name_authorisation_removes_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let authorisation_id = NameAuthorisationId::new();
        let name_authorisation = sample_name_authorisation(authorisation_id);

        name_authorisation.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();

        let response = delete_name_authorisation(
            NameAuthorisationDeletePath { authorisation_id },
            name_authorisation,
            context,
            store.clone(),
            Form(EmptyForm::new(csrf_token)),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(
            location,
            NameAuthorisation::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let name_authorisations = store.get_name_authorisations();
        assert!(name_authorisations.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn delete_name_authorisation_invalid_csrf_error_page() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let authorisation_id = NameAuthorisationId::new();
        let name_authorisation = sample_name_authorisation(authorisation_id);
        name_authorisation.create(&store).await?;

        let context = Context::new_test_without_db();

        let response = delete_name_authorisation(
            NameAuthorisationDeletePath { authorisation_id },
            name_authorisation.clone(),
            context,
            store.clone(),
            Form(EmptyForm::new(TokenValue("invalid".to_string()))),
        )
        .await
        .unwrap_err();

        assert!(matches!(response, AppError::CsrfTokenInvalid));

        let name_authorisations = store.get_name_authorisations();
        assert_eq!(name_authorisations.len(), 1);

        Ok(())
    }
}
