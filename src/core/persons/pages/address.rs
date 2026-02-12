use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    AppError, AppResponse, AppStore, Context, Form, HtmlTemplate, filters,
    form::FormData,
    persons::{AddressForm, InitialQuery, Person, pages::UpdatePersonAddressPath},
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
    Query(query): Query<InitialQuery>,
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
    Query(query): Query<InitialQuery>,
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
            person
                .update_address(&store, person.address.clone())
                .await?;

            Ok(Redirect::to(&Person::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, DutchAddressForm, Form,
        persons::PersonId,
        test_utils::{response_body_string, sample_address_form, sample_person},
    };
    use axum::{
        extract::Query,
        http::{StatusCode, header},
        response::IntoResponse,
    };

    #[tokio::test]
    async fn update_person_address_renders_existing_person() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let person_id: PersonId = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let response = update_person_address(
            UpdatePersonAddressPath { person_id },
            Context::new_test_without_db(),
            person,
            Query(InitialQuery::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Juinen"));

        Ok(())
    }

    #[tokio::test]
    async fn update_person_address_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
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
            Query(InitialQuery::default()),
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
        assert_eq!(
            updated.address.locality.as_deref().map(|v| v.to_string()),
            Some("Juinen".to_string())
        );

        Ok(())
    }

    #[tokio::test]
    async fn update_person_address_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.address.postal_code = "a".to_string();

        let response = update_person_address_submit(
            UpdatePersonAddressPath { person_id },
            context,
            person,
            State(store),
            Query(InitialQuery::default()),
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

    #[tokio::test]
    async fn update_person_address_dutch_xor_non_dutch() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
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
            Query(InitialQuery::default()),
            Form(AddressForm {
                address: DutchAddressForm {
                    locality: "Juinen".to_string(),
                    postal_code: "1234 AB".to_string(),
                    house_number: "10".to_string(),
                    house_number_addition: "A".to_string(),
                    street_name: "Stationsstraat".to_string(),
                },
                csrf_token: context.csrf_tokens.issue().value,
            }),
        )
        .await
        .unwrap();

        // The international address should be removed because `is_dutch` is true
        let updated = store.get_person(person_id)?;
        assert_eq!(
            updated.address.locality.as_deref().map(|v| v.to_string()),
            Some("Juinen".to_string())
        );
        assert_eq!(
            updated.address.postal_code.unwrap(),
            "1234AB".parse().unwrap()
        );
        assert_eq!(
            updated
                .address
                .house_number
                .as_deref()
                .map(|v| v.to_string()),
            Some("10".to_string())
        );
        assert_eq!(
            updated
                .address
                .house_number_addition
                .as_deref()
                .map(|v| v.to_string()),
            Some("A".to_string())
        );
        assert_eq!(
            updated
                .address
                .street_name
                .as_deref()
                .map(|v| v.to_string()),
            Some("Stationsstraat".to_string())
        );

        Ok(())
    }
}
