use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, Context, CsrfTokens, DbConnection, HtmlTemplate, filters,
    form::{FormData, Validate},
    persons::{self, AddressForm, Person, pages::EditPersonAddressPath},
    t,
};

#[derive(Template)]
#[template(path = "persons/address.html")]
struct PersonAddressUpdateTemplate {
    person: Person,
    form: FormData<AddressForm>,
}

pub async fn edit_person_address(
    _: EditPersonAddressPath,
    context: Context,
    person: Person,
    csrf_tokens: CsrfTokens,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        PersonAddressUpdateTemplate {
            form: FormData::new_with_data(AddressForm::from(person.clone()), &csrf_tokens),
            person,
        },
        context,
    ))
}

pub async fn update_person_address(
    _: EditPersonAddressPath,
    context: Context,
    person: Person,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    form: Form<AddressForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&person, &csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonAddressUpdateTemplate {
                person,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(mut person) => {
            person.normalize_address();
            persons::repository::update_address(&mut conn, &person).await?;

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
        Context, CsrfTokens, DbConnection, Locale,
        persons::{self, PersonId},
        test_utils::{response_body_string, sample_address_form, sample_person},
    };

    #[sqlx::test]
    async fn edit_person_address_renders_existing_person(pool: PgPool) -> Result<(), sqlx::Error> {
        let person_id: PersonId = PersonId::new();
        let person = sample_person(person_id);

        let mut conn = pool.acquire().await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let response = edit_person_address(
            EditPersonAddressPath { person_id },
            Context::new(Locale::En),
            person,
            CsrfTokens::default(),
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
    async fn update_person_address_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        let mut conn = pool.acquire().await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let form = sample_address_form(&csrf_token);

        let response = update_person_address(
            EditPersonAddressPath { person_id },
            Context::new(Locale::En),
            person,
            csrf_tokens,
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

        assert_eq!(location, Person::list_path());

        let mut conn = pool.acquire().await?;
        let updated = persons::repository::get_person(&mut conn, person_id)
            .await?
            .expect("updated person");
        assert_eq!(updated.locality, Some("Juinen".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_address_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        let mut conn = pool.acquire().await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.postal_code = "a".to_string();

        let response = update_person_address(
            EditPersonAddressPath { person_id },
            Context::new(Locale::En),
            person,
            csrf_tokens,
            DbConnection(pool.acquire().await?),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("The value is too short"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_address_dutch_xor_non_dutch(pool: PgPool) -> Result<(), sqlx::Error> {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        let mut conn = pool.acquire().await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let csrf_tokens = CsrfTokens::default();

        // Update with Dutch address (but all form fields filled)
        update_person_address(
            EditPersonAddressPath { person_id },
            Context::new(Locale::En),
            person.clone(),
            csrf_tokens.clone(),
            DbConnection(pool.acquire().await?),
            Form(AddressForm {
                locality: "Juinen".to_string(),
                postal_code: "1234 AB".to_string(),
                house_number: "10".to_string(),
                house_number_addition: "A".to_string(),
                street_name: "Stationsstraat".to_string(),
                custom_country: "Netherlands".to_string(),
                custom_region: "Noord Holland".to_string(),
                address_line_1: "Stationsstraat 10A".to_string(),
                address_line_2: "1234AB Juinen".to_string(),
                is_dutch: "true".to_string(),
                csrf_token: csrf_tokens.issue().value,
            }),
        )
        .await
        .unwrap();

        // The international address should be removed because `is_dutch` is true
        let mut conn = pool.acquire().await?;
        let updated = persons::repository::get_person(&mut conn, person_id)
            .await?
            .expect("updated person");
        assert_eq!(updated.is_dutch, Some(true));
        assert_eq!(updated.locality, Some("Juinen".to_string()));
        assert_eq!(updated.postal_code, Some("1234 AB".to_string()));
        assert_eq!(updated.house_number, Some("10".to_string()));
        assert_eq!(updated.house_number_addition, Some("A".to_string()));
        assert_eq!(updated.street_name, Some("Stationsstraat".to_string()));
        assert_eq!(updated.custom_country, None);
        assert_eq!(updated.custom_region, None);
        assert_eq!(updated.address_line_1, None);
        assert_eq!(updated.address_line_2, None);

        // Update with non-Dutch address (but all form fields filled)
        update_person_address(
            EditPersonAddressPath { person_id },
            Context::new(Locale::En),
            updated.clone(),
            csrf_tokens.clone(),
            DbConnection(pool.acquire().await?),
            Form(AddressForm {
                locality: "Juinen".to_string(),
                postal_code: "1234 AB".to_string(),
                house_number: "10".to_string(),
                house_number_addition: "A".to_string(),
                street_name: "Stationsstraat".to_string(),
                custom_country: "Netherlands".to_string(),
                custom_region: "Noord Holland".to_string(),
                address_line_1: "Stationsstraat 10A".to_string(),
                address_line_2: "1234AB Juinen".to_string(),
                is_dutch: "false".to_string(),
                csrf_token: csrf_tokens.issue().value,
            }),
        )
        .await
        .unwrap();

        // The Dutch address should be removed because `is_dutch` is false
        let mut conn = pool.acquire().await?;
        let updated = persons::repository::get_person(&mut conn, person_id)
            .await?
            .expect("updated person");
        assert_eq!(updated.is_dutch, Some(false));
        assert_eq!(updated.locality, None);
        assert_eq!(updated.postal_code, None);
        assert_eq!(updated.house_number, None);
        assert_eq!(updated.house_number_addition, None);
        assert_eq!(updated.street_name, None);
        assert_eq!(updated.custom_country, Some("Netherlands".to_string()));
        assert_eq!(updated.custom_region, Some("Noord Holland".to_string()));
        assert_eq!(
            updated.address_line_1,
            Some("Stationsstraat 10A".to_string())
        );
        assert_eq!(updated.address_line_2, Some("1234AB Juinen".to_string()));

        Ok(())
    }
}
