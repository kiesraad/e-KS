use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AppError, AppResponse, Context, Form, HtmlTemplate, Store, filters,
    form::FormData,
    persons::{Person, PersonForm, pages::UpdatePersonPath},
};

#[derive(Template)]
#[template(path = "persons/pages/update.html")]
struct PersonUpdateTemplate {
    person: Person,
    on_candidate_lists: usize,
    form: FormData<PersonForm>,
}

pub async fn update_person(
    _: UpdatePersonPath,
    context: Context,
    State(store): State<Store>,
    person: Person,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        PersonUpdateTemplate {
            form: FormData::new_with_data(PersonForm::from(person.clone()), &context.csrf_tokens),
            on_candidate_lists: store.count_candidate_lists(person.id)?,
            person,
        },
        context,
    ))
}

pub async fn update_person_submit(
    _: UpdatePersonPath,
    context: Context,
    State(store): State<Store>,
    person: Person,
    Form(form): Form<PersonForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&person, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonUpdateTemplate {
                on_candidate_lists: store.count_candidate_lists(person.id)?,
                person,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            person.update(&store).await?;

            Ok(Redirect::to(&person.after_update_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, Context, Form, Store,
        persons::PersonId,
        test_utils::{response_body_string, sample_person, sample_person_form},
    };
    use axum::{
        http::{StatusCode, header},
        response::IntoResponse,
    };

    #[tokio::test]
    async fn update_person_renders_existing_person() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let response = update_person(
            UpdatePersonPath { person_id },
            Context::new_test_without_db(),
            State(store),
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

    #[tokio::test]
    async fn update_person_persists_and_redirects() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.name.last_name = "Updated".to_string();
        let expected_path = person.after_update_path();

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
        assert_eq!(location, expected_path);

        let updated = store.get_person(person_id)?;
        assert_eq!(updated.name.last_name.to_string(), "Updated");

        Ok(())
    }

    #[tokio::test]
    async fn update_person_invalid_form_renders_template() -> Result<(), AppError> {
        let store = Store::new_for_test().await;
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_person_form(&csrf_token);
        form.name.last_name = " ".to_string();

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
