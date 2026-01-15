use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate, filters,
    form::{FormData, Validate},
    pagination::{Pagination, SortDirection},
    persons::{
        self, AddressForm, Person, PersonSort,
        pages::{EditPersonAddressPath, person_not_found},
    },
    t,
};

#[derive(Template)]
#[template(path = "persons/address.html")]
struct PersonAddressUpdateTemplate {
    person: Person,
    form: FormData<AddressForm>,
}

pub async fn edit_person_address(
    EditPersonAddressPath { id }: EditPersonAddressPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> AppResponse<impl IntoResponse> {
    let person = persons::repository::get_person(&mut conn, &id)
        .await?
        .ok_or(person_not_found(id, context.locale))?;

    Ok(HtmlTemplate(
        PersonAddressUpdateTemplate {
            form: FormData::new_with_data(AddressForm::from(person.clone()), &csrf_tokens),
            person,
        },
        context,
    ))
}

pub async fn update_person_address(
    EditPersonAddressPath { id }: EditPersonAddressPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
    form: Form<AddressForm>,
) -> Result<Response, AppError> {
    let person = persons::repository::get_person(&mut conn, &id)
        .await?
        .ok_or(person_not_found(id, context.locale))?;

    match form.validate(Some(&person), app_state.csrf_tokens()) {
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
            persons::repository::update_person(&mut conn, &person).await?;

            // Redirect to the persons list after updating, sorted by updated, so the updated person is visible at the top
            let pagination = Pagination {
                sort: PersonSort::UpdatedAt,
                order: SortDirection::Desc,
                ..Default::default()
            };

            Ok(Redirect::to(&Person::list_path_with_pagination(&pagination)).into_response())
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
    use uuid::Uuid;

    use crate::{
        AppState, Context, CsrfTokens, DbConnection, Locale, persons,
        test_utils::{response_body_string, sample_address_form, sample_person},
    };

    #[sqlx::test]
    async fn edit_person_address_renders_existing_person(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let response = edit_person_address(
            EditPersonAddressPath { id },
            Context::new(Locale::En),
            CsrfTokens::default(),
            DbConnection(pool.acquire().await?),
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
        let id = Uuid::new_v4();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_token = app_state.csrf_tokens().issue().value;
        let form = sample_address_form(&csrf_token);

        let response = update_person_address(
            EditPersonAddressPath { id },
            Context::new(Locale::En),
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

        let pagination = Pagination {
            sort: PersonSort::UpdatedAt,
            order: SortDirection::Desc,
            ..Default::default()
        };
        assert_eq!(location, Person::list_path_with_pagination(&pagination));

        let mut conn = pool.acquire().await?;
        let updated = persons::repository::get_person(&mut conn, &id)
            .await?
            .expect("updated person");
        assert_eq!(updated.locality, Some("Juinen".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_address_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_token = app_state.csrf_tokens().issue().value;
        let mut form = sample_address_form(&csrf_token);
        form.postal_code = "a".to_string();

        let response = update_person_address(
            EditPersonAddressPath { id },
            Context::new(Locale::En),
            State(app_state),
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
        let id = Uuid::new_v4();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        persons::repository::create_person(&mut conn, &person).await?;

        let app_state = AppState::new_for_tests(pool.clone());

        // Update with Dutch address (but all form fields filled)
        update_person_address(
            EditPersonAddressPath { id },
            Context::new(Locale::En),
            State(app_state.clone()),
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
                csrf_token: app_state.csrf_tokens().issue().value,
            }),
        )
        .await
        .unwrap();

        // The international address should be removed because `is_dutch` is true
        let mut conn = pool.acquire().await?;
        let updated = persons::repository::get_person(&mut conn, &id)
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
            EditPersonAddressPath { id },
            Context::new(Locale::En),
            State(app_state.clone()),
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
                csrf_token: app_state.csrf_tokens().issue().value,
            }),
        )
        .await
        .unwrap();

        // The Dutch address should be removed because `is_dutch` is false
        let mut conn = pool.acquire().await?;
        let updated = persons::repository::get_person(&mut conn, &id)
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
