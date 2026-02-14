use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate, filters,
    form::FormData,
    persons::{Person, PersonForm, pages::PersonsCreatePath},
};

#[derive(Template)]
#[template(path = "persons/create.html")]
struct PersonCreateTemplate {
    form: FormData<PersonForm>,
}

pub async fn create_person(
    _: PersonsCreatePath,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        PersonCreateTemplate {
            form: FormData::new(&context.csrf_tokens),
        },
        context,
    ))
}

pub async fn create_person_submit(
    _: PersonsCreatePath,
    context: Context,
    State(store): State<AppStore>,
    Form(form): Form<PersonForm>,
) -> Result<Response, AppError> {
    match form.validate_create_unique(&context.csrf_tokens, &store) {
        Err(form_data) => {
            Ok(HtmlTemplate(PersonCreateTemplate { form: *form_data }, context).into_response())
        }
        Ok(person) => {
            person.create(&store).await?;

            Ok(Redirect::to(&person.after_create_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, Form,
        test_utils::{response_body_string, sample_person_form},
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };

    #[tokio::test]
    async fn create_person_renders_csrf_field() {
        let context = Context::new_test_without_db();

        let response = create_person(PersonsCreatePath {}, context)
            .await
            .unwrap()
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
    }

    #[tokio::test]
    async fn create_person_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_person_form(&csrf_token);

        let response = create_person_submit(
            PersonsCreatePath {},
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
        let persons = store.get_persons()?;
        assert_eq!(persons.len(), 1);
        let created = persons.first().expect("person");
        assert_eq!(location, created.after_create_path());

        let count = store.get_person_count()?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[tokio::test]
    async fn create_person_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.name.last_name = " ".to_string();

        let response =
            create_person_submit(PersonsCreatePath {}, context, State(store), Form(form))
                .await
                .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("This field must not be empty."));

        Ok(())
    }

    #[tokio::test]
    async fn create_person_duplicate_name_renders_error() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let existing = crate::test_utils::sample_person(crate::persons::PersonId::new());
        existing.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_person_form(&csrf_token);

        let response =
            create_person_submit(PersonsCreatePath {}, context, State(store), Form(form))
                .await
                .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("A person with this name already exists."));

        Ok(())
    }
}
