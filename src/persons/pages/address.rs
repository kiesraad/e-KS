use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, AppStore, Context, HtmlTemplate, filters,
    form::{FormData, Validate},
    persons::{AddressForm, InitialEditQuery, Person, pages::UpdatePersonAddressPath},
};

#[derive(Template)]
#[template(path = "persons/update_address.html")]
struct PersonAddressUpdateTemplate {
    should_warn: bool,
    person: Person,
    form: FormData<AddressForm>,
}

pub async fn update_person_address(
    _: UpdatePersonAddressPath,
    context: Context,
    person: Person,
    Query(query): Query<InitialEditQuery>,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        PersonAddressUpdateTemplate {
            should_warn: query.should_warn(),
            form: FormData::new_with_data(AddressForm::from(person.clone()), &context.csrf_tokens),
            person,
        },
        context,
    ))
}

pub async fn update_person_address_submit(
    _: UpdatePersonAddressPath,
    context: Context,
    person: Person,
    State(store): State<AppStore>,
    Query(query): Query<InitialEditQuery>,
    Form(form): Form<AddressForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&person, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonAddressUpdateTemplate {
                person,
                should_warn: query.should_warn(),
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            person.update_address(&store).await?;

            Ok(Redirect::to(&Person::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::Query,
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        AppError, AppStore, Context,
        persons::PersonId,
        test_utils::{response_body_string, sample_address_form, sample_person},
    };

    #[sqlx::test]
    async fn update_person_address_renders_existing_person(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let person_id: PersonId = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let response = update_person_address(
            UpdatePersonAddressPath { person_id },
            Context::new_test_without_db(),
            person,
            Query(InitialEditQuery::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Juinen"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_address_persists_and_redirects(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_address_form(&csrf_token);

        let response = update_person_address_submit(
            UpdatePersonAddressPath { person_id },
            context,
            person,
            State(store.clone()),
            Query(InitialEditQuery::default()),
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

        assert_eq!(location, Person::list_path());

        let updated = store.get_person(person_id)?;
        assert_eq!(updated.locality, Some("Juinen".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_address_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.postal_code = "a".to_string();

        let response = update_person_address_submit(
            UpdatePersonAddressPath { person_id },
            context,
            person,
            State(store),
            Query(InitialEditQuery::default()),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("The postal code is not valid"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_address_dutch_xor_non_dutch(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();

        // Update with Dutch address (but all form fields filled)
        update_person_address_submit(
            UpdatePersonAddressPath { person_id },
            context.clone(),
            person.clone(),
            State(store.clone()),
            Query(InitialEditQuery::default()),
            Form(AddressForm {
                locality: "Juinen".to_string(),
                postal_code: "1234 AB".to_string(),
                house_number: "10".to_string(),
                house_number_addition: "A".to_string(),
                street_name: "Stationsstraat".to_string(),
                csrf_token: context.csrf_tokens.issue().value,
            }),
        )
        .await
        .unwrap();

        // The international address should be removed because `is_dutch` is true
        let updated = store.get_person(person_id)?;
        assert_eq!(updated.locality, Some("Juinen".to_string()));
        assert_eq!(updated.postal_code, Some("1234AB".to_string()));
        assert_eq!(updated.house_number, Some("10".to_string()));
        assert_eq!(updated.house_number_addition, Some("A".to_string()));
        assert_eq!(updated.street_name, Some("Stationsstraat".to_string()));

        Ok(())
    }
}
