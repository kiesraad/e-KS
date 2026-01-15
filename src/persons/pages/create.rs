use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate, filters,
    form::{FormData, Validate},
    persons::{self, Person, PersonForm, pages::PersonsNewPath},
    t,
};

#[derive(Template)]
#[template(path = "persons/create.html")]
struct PersonCreateTemplate {
    form: FormData<PersonForm>,
}

pub async fn new_person_form(
    _: PersonsNewPath,
    context: Context,
    csrf_tokens: CsrfTokens,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        PersonCreateTemplate {
            form: FormData::new(&csrf_tokens),
        },
        context,
    )
    .into_response())
}

pub async fn create_person(
    _: PersonsNewPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
    form: Form<PersonForm>,
) -> Result<Response, AppError> {
    match form.validate(None, app_state.csrf_tokens()) {
        Err(form_data) => {
            Ok(HtmlTemplate(PersonCreateTemplate { form: form_data }, context).into_response())
        }
        Ok(person) => {
            persons::repository::create_person(&mut conn, &person).await?;

            // Redirect to the address edit page
            Ok(Redirect::to(&person.edit_address_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::State,
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        AppState, Context, CsrfTokens, DbConnection, Locale, persons,
        test_utils::{response_body_string, sample_person_form},
    };

    #[tokio::test]
    async fn new_person_form_renders_csrf_field() {
        let context = Context::new(Locale::En);
        let csrf_tokens = CsrfTokens::default();

        let response = new_person_form(PersonsNewPath {}, context, csrf_tokens)
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
        let app_state = AppState::new_for_tests(pool.clone());
        let context = Context::new(Locale::En);
        let csrf_token = app_state.csrf_tokens().issue().value;
        let form = sample_person_form(&csrf_token);

        let response = create_person(
            PersonsNewPath {},
            context,
            State(app_state),
            DbConnection(pool.acquire().await?),
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
        let count = persons::repository::count_persons(&mut conn).await?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn create_person_invalid_form_renders_template(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let context = Context::new(Locale::En);
        let csrf_token = app_state.csrf_tokens().issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.last_name = " ".to_string();

        let response = create_person(
            PersonsNewPath {},
            context,
            State(app_state),
            DbConnection(pool.acquire().await?),
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
