use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, AppResponse, AppStore, Context, Form, HtmlTemplate, filters,
    form::{EmptyForm, FormData},
    persons::{Person, pages::DeletePersonPath},
    redirect_success,
};

#[derive(Template)]
#[template(path = "persons/pages/delete.html")]
struct DeletePersonTemplate {
    person: Person,
    form: FormData<EmptyForm>,
}

pub async fn delete_person_confirm(
    _: DeletePersonPath,
    context: Context,
    person: Person,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        DeletePersonTemplate {
            form: FormData::new(&context.session.csrf_token),
            person,
        },
        context,
    ))
}

pub async fn delete_person(
    _: DeletePersonPath,
    context: Context,
    person: Person,
    store: AppStore,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.session.csrf_token) {
        Err(_) => Err(AppError::CsrfTokenInvalid),
        Ok(_) => {
            person.delete(&store).await?;

            Ok(redirect_success(Person::list_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum_extra::routing::TypedPath;

    use super::*;
    use crate::{
        AppError, AppStore, Context, Form, QueryParamState,
        persons::PersonId,
        test_utils::{response_body_string, sample_person},
    };

    #[tokio::test]
    async fn delete_person_confirm_contains_delete_button() -> Result<(), AppError> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        let response = delete_person_confirm(
            DeletePersonPath { person_id },
            Context::new_test_without_db(),
            person.clone(),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&person.delete_path().to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn delete_person_removes_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_token.clone();

        let response = delete_person(
            DeletePersonPath { person_id },
            context,
            person,
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
            Person::list_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let found = store.get_person(person_id);
        assert!(found.is_err());

        Ok(())
    }
}
