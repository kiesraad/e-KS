use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    persons::{
        self, COUNTRY_CODES, Person, PersonForm, PersonPagination, PersonSort,
        pages::PersonsNewPath,
    },
};

#[derive(Template)]
#[template(path = "persons/create.html")]
struct PersonCreateTemplate {
    form: FormData<PersonForm>,
    person_pagination: PersonPagination,
    countries: &'static [&'static str],
}

pub async fn new_person_form(
    _: PersonsNewPath,
    context: Context,
    person_pagination: PersonPagination,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        PersonCreateTemplate {
            form: FormData::new(&context.csrf_tokens),
            person_pagination,
            countries: &COUNTRY_CODES,
        },
        context,
    ))
}

pub async fn create_person(
    _: PersonsNewPath,
    context: Context,
    State(pool): State<PgPool>,
    person_pagination: PersonPagination,
    Form(form): Form<PersonForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonCreateTemplate {
                form: form_data,
                person_pagination,
                countries: &COUNTRY_CODES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            persons::create_person(&pool, &person).await?;

            Ok(Redirect::to(&person.edit_address_path()).into_response())
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
        Context, persons,
        test_utils::{response_body_string, sample_person_form},
    };

    #[sqlx::test]
    async fn new_person_form_renders_csrf_field(pool: PgPool) {
        let context = Context::new_test(pool).await;

        let response = new_person_form(PersonsNewPath {}, context, PersonPagination::empty())
            .await
            .unwrap()
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains("action=\"/persons/new\""));
    }

    #[sqlx::test]
    async fn create_person_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_person_form(&csrf_token);

        let response = create_person(
            PersonsNewPath {},
            context,
            State(pool.clone()),
            PersonPagination::empty(),
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
        assert!(location.ends_with("/address"));

        let count = persons::count_persons(&pool).await?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn create_person_invalid_form_renders_template(pool: PgPool) -> Result<(), sqlx::Error> {
        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = create_person(
            PersonsNewPath {},
            context,
            State(pool.clone()),
            PersonPagination::empty(),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("This field must not be empty."));

        Ok(())
    }
}
