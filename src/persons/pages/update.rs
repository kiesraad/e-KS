use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, AppStore, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    persons::{COUNTRY_CODES, Person, PersonForm, pages::UpdatePersonPath},
};

#[derive(Template)]
#[template(path = "persons/update.html")]
struct PersonUpdateTemplate {
    person: Person,
    form: FormData<PersonForm>,
    countries: &'static [&'static str],
}

pub async fn update_person(
    _: UpdatePersonPath,
    context: Context,
    person: Person,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        PersonUpdateTemplate {
            form: FormData::new_with_data(PersonForm::from(person.clone()), &context.csrf_tokens),
            person,
            countries: &COUNTRY_CODES,
        },
        context,
    ))
}

pub async fn update_person_submit(
    _: UpdatePersonPath,
    context: Context,
    State(store): State<AppStore>,
    person: Person,
    Form(form): Form<PersonForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&person, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonUpdateTemplate {
                person,
                form: form_data,
                countries: &COUNTRY_CODES,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            person.update(&store).await?;

            Ok(Redirect::to(&Person::list_path()).into_response())
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
        persons::PersonId,
        test_utils::{response_body_string, sample_person, sample_person_form},
    };

    #[sqlx::test]
    async fn update_person_renders_existing_person(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let response = update_person(
            UpdatePersonPath { person_id },
            Context::new_test_without_db(),
            person,
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
    async fn update_person_persists_and_redirects(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = "Updated".to_string();

        let response = update_person_submit(
            UpdatePersonPath { person_id },
            context,
            State(store.clone()),
            person,
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
        assert!(location.ends_with("/persons"));

        let updated = store.get_person(person_id)?;
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_invalid_form_renders_template(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = update_person_submit(
            UpdatePersonPath { person_id },
            context,
            State(store),
            person,
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
