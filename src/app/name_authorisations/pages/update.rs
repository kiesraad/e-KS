use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate,
    common::Problematic,
    filters,
    form::FormData,
    name_authorisations::{NameAuthorisation, NameAuthorisationForm},
    redirect_success,
};

use super::NameAuthorisationUpdatePath;

#[derive(Template)]
#[template(path = "name_authorisations/pages/update.html")]
struct NameAuthorisationUpdateTemplate {
    name_authorisation: NameAuthorisation,
    form: FormData<NameAuthorisationForm>,
}

pub async fn update_name_authorisation(
    _: NameAuthorisationUpdatePath,
    context: Context,
    name_authorisation: NameAuthorisation,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(
        NameAuthorisationUpdateTemplate {
            form: FormData::new_with_data(
                name_authorisation.clone().into(),
                &context.session.csrf_token,
            ),
            name_authorisation,
        },
        context,
    )
    .into_response())
}

pub async fn update_name_authorisation_submit(
    _: NameAuthorisationUpdatePath,
    context: Context,
    name_authorisation: NameAuthorisation,
    store: AppStore,
    Form(form): Form<NameAuthorisationForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&name_authorisation, &context.session.csrf_token) {
        Err(form_data) => Ok(HtmlTemplate(
            NameAuthorisationUpdateTemplate {
                name_authorisation,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(name_authorisation) => {
            name_authorisation.update(&store).await?;

            Ok(redirect_success(NameAuthorisation::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState,
        name_authorisations::NameAuthorisationId,
        test_utils::{
            response_body_string, sample_name_authorisation, sample_name_authorisation_form,
        },
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::routing::TypedPath;

    #[tokio::test]
    async fn update_name_authorisation_renders_existing_agent() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let authorisation_id = NameAuthorisationId::new();
        let name_authorisation = sample_name_authorisation(authorisation_id);

        name_authorisation.create(&store).await?;

        let response = update_name_authorisation(
            NameAuthorisationUpdatePath { authorisation_id },
            Context::new_test_without_db(),
            name_authorisation.clone(),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(name_authorisation.name.last_name.as_str()));

        Ok(())
    }

    #[tokio::test]
    async fn update_name_authorisation_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let authorisation_id = NameAuthorisationId::new();
        let name_authorisation = sample_name_authorisation(authorisation_id);
        name_authorisation.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();
        let mut form = sample_name_authorisation_form(&csrf_token);
        form.name.last_name = "Updated".to_string();

        let response = update_name_authorisation_submit(
            NameAuthorisationUpdatePath { authorisation_id },
            context,
            name_authorisation.clone(),
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
        assert_eq!(
            location,
            NameAuthorisation::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let updated = store.get_name_authorisation(authorisation_id)?;
        assert_eq!(updated.name.last_name.to_string(), "Updated");

        Ok(())
    }

    #[tokio::test]
    async fn update_name_authorisation_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let authorisation_id = NameAuthorisationId::new();
        let name_authorisation = sample_name_authorisation(authorisation_id);
        name_authorisation.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();
        let mut form = sample_name_authorisation_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response = update_name_authorisation_submit(
            NameAuthorisationUpdatePath { authorisation_id },
            context,
            name_authorisation.clone(),
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
