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
        pages::{EditPersonAddressPath, person_not_found},
        repository,
        structs::{AddressForm, Person, PersonSort},
    },
    t,
};

#[derive(Template)]
#[template(path = "persons/address.html")]
struct PersonAddressUpdateTemplate {
    person: Person,
    form: FormData<AddressForm>,
}

pub(crate) async fn edit_person_address_form(
    EditPersonAddressPath { id }: EditPersonAddressPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> AppResponse<impl IntoResponse> {
    let person = repository::get_person(&mut conn, &id)
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

pub(crate) async fn update_person_address(
    EditPersonAddressPath { id }: EditPersonAddressPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
    form: Form<AddressForm>,
) -> Result<Response, AppError> {
    let person = repository::get_person(&mut conn, &id)
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
        Ok(person) => {
            repository::update_person(&mut conn, &person).await?;

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
