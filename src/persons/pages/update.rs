use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, Context, DbConnection, HtmlTemplate, filters,
    form::{FormData, Validate},
    persons::{
        self, COUNTRY_CODES, Person, PersonForm, PersonPagination, PersonSort,
        pages::EditPersonPath,
    },
};

#[derive(Template)]
#[template(path = "persons/update.html")]
struct PersonUpdateTemplate {
    person: Person,
    person_pagination: PersonPagination,
    form: FormData<PersonForm>,
    countries: &'static [&'static str],
}

pub async fn edit_person_form(
    _: EditPersonPath,
    context: Context,
    person: Person,
    person_pagination: PersonPagination,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        PersonUpdateTemplate {
            form: FormData::new_with_data(PersonForm::from(person.clone()), &context.csrf_tokens),
            person,
            person_pagination,
            countries: &COUNTRY_CODES,
        },
        context,
    ))
}

pub async fn update_person(
    _: EditPersonPath,
    context: Context,
    DbConnection(mut conn): DbConnection,
    person: Person,
    person_pagination: PersonPagination,
    form: Form<PersonForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&person, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonUpdateTemplate {
                person,
                person_pagination,
                form: form_data,
                countries: &COUNTRY_CODES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            persons::update_person(&mut conn, &person).await?;

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
        Context, DbConnection,
        persons::{self, PersonId},
        test_utils::{response_body_string, sample_person, sample_person_form},
    };

    #[sqlx::test]
    async fn edit_person_form_renders_existing_person(pool: PgPool) -> Result<(), sqlx::Error> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        let mut conn = pool.acquire().await?;
        persons::create_person(&mut conn, &person).await?;

        let response = edit_person_form(
            EditPersonPath { person_id },
            Context::new_test(),
            person,
            PersonPagination::empty(),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        let mut conn = pool.acquire().await?;
        persons::create_person(&mut conn, &person).await?;

        let context = Context::new_test();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update_person(
            EditPersonPath { person_id },
            context,
            DbConnection(pool.acquire().await?),
            person,
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

        let mut conn = pool.acquire().await?;
        let updated = persons::get_person(&mut conn, person_id)
            .await?
            .expect("updated person");
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_invalid_form_renders_template(pool: PgPool) -> Result<(), sqlx::Error> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        let mut conn = pool.acquire().await?;
        persons::create_person(&mut conn, &person).await?;

        let context = Context::new_test();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update_person(
            EditPersonPath { person_id },
            context,
            DbConnection(pool.acquire().await?),
            person,
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
