use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, Context, DbConnection, HtmlTemplate, filters,
    form::{FormData, Validate},
    persons::{
        self, AddressForm, Person, PersonPagination, PersonSort, pages::EditPersonAddressPath,
    },
    t,
};

#[derive(Template)]
#[template(path = "persons/address.html")]
struct PersonAddressUpdateTemplate {
    person: Person,
    form: FormData<AddressForm>,
    person_pagination: PersonPagination,
}

pub async fn edit_person_address(
    _: EditPersonAddressPath,
    context: Context,
    person: Person,
    person_pagination: PersonPagination,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        PersonAddressUpdateTemplate {
            form: FormData::new_with_data(AddressForm::from(person.clone()), &context.csrf_tokens),
            person,
            person_pagination,
        },
        context,
    ))
}

pub async fn update_person_address(
    _: EditPersonAddressPath,
    context: Context,
    person: Person,
    DbConnection(mut conn): DbConnection,
    person_pagination: PersonPagination,
    form: Form<AddressForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&person, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonAddressUpdateTemplate {
                person,
                form: form_data,
                person_pagination,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            persons::update_address(&mut conn, &person).await?;

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
        Context, DbConnection,
        persons::{self, PersonId},
        test_utils::{response_body_string, sample_address_form, sample_person},
    };

    #[sqlx::test]
    async fn edit_person_address_renders_existing_person(pool: PgPool) -> Result<(), sqlx::Error> {
        let person_id: PersonId = PersonId::new();
        let person = sample_person(person_id);

        let mut conn = pool.acquire().await?;
        persons::create_person(&mut conn, &person).await?;

        let response = edit_person_address(
            EditPersonAddressPath { person_id },
            Context::new_test(),
            person,
            PersonPagination::empty(),
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
        persons::create_person(&mut conn, &person).await?;

        let context = Context::new_test();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_address_form(&csrf_token);

        let response = update_person_address(
            EditPersonAddressPath { person_id },
            context,
            person,
            DbConnection(pool.acquire().await?),
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

        assert_eq!(location, Person::list_path());

        let mut conn = pool.acquire().await?;
        let updated = persons::get_person(&mut conn, person_id)
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
        persons::create_person(&mut conn, &person).await?;

        let context = Context::new_test();
        let csrf_token = context.csrf_tokens.issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.postal_code = "a".to_string();

        let response = update_person_address(
            EditPersonAddressPath { person_id },
            context,
            person,
            DbConnection(pool.acquire().await?),
            PersonPagination::empty(),
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
        persons::create_person(&mut conn, &person).await?;

        let context = Context::new_test();

        // Update with Dutch address (but all form fields filled)
        update_person_address(
            EditPersonAddressPath { person_id },
            context.clone(),
            person.clone(),
            DbConnection(pool.acquire().await?),
            PersonPagination::empty(),
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
        let mut conn = pool.acquire().await?;
        let updated = persons::get_person(&mut conn, person_id)
            .await?
            .expect("updated person");
        assert_eq!(updated.locality, Some("Juinen".to_string()));
        assert_eq!(updated.postal_code, Some("1234 AB".to_string()));
        assert_eq!(updated.house_number, Some("10".to_string()));
        assert_eq!(updated.house_number_addition, Some("A".to_string()));
        assert_eq!(updated.street_name, Some("Stationsstraat".to_string()));

        Ok(())
    }
}
