use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    persons::{COUNTRY_CODES, Person, PersonForm, pages::PersonsCreatePath},
};

#[derive(Template)]
#[template(path = "persons/create.html")]
struct PersonCreateTemplate {
    form: FormData<PersonForm>,
    countries: &'static [&'static str],
}

pub async fn create_person(
    _: PersonsCreatePath,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        PersonCreateTemplate {
            form: FormData::new(&context.csrf_tokens),
            countries: &COUNTRY_CODES,
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
    match form.validate_create(&context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonCreateTemplate {
                form: form_data,
                countries: &COUNTRY_CODES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            person.create(&store).await?;

            Ok(Redirect::to(&person.after_create_path()).into_response())
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
        test_utils::{response_body_string, sample_person_form},
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
        assert!(body.contains("action=\"/persons/create\""));
    }

    #[sqlx::test]
    async fn create_person_persists_and_redirects(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
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
        assert!(location.contains("/address"));

        let count = store.get_person_count()?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn create_person_invalid_form_renders_template(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = " ".to_string();

        let response =
            create_person_submit(PersonsCreatePath {}, context, State(store), Form(form))
                .await
                .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("This field must not be empty."));

        Ok(())
    }
}
