use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate, filters,
    form::{FormData, Validate},
    persons::{
        pages::person_not_found,
        repository,
        structs::{Person, PersonForm},
    },
    t,
};

use super::EditPersonPath;

#[derive(Template)]
#[template(path = "persons/update.html")]
struct PersonUpdateTemplate {
    person: Person,
    form: FormData<PersonForm>,
}

pub(crate) async fn edit_person_form(
    EditPersonPath { id }: EditPersonPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> AppResponse<impl IntoResponse> {
    let person = repository::get_person(&mut conn, &id)
        .await?
        .ok_or(person_not_found(id, context.locale))?;

    Ok(HtmlTemplate(
        PersonUpdateTemplate {
            form: FormData::new_with_data(PersonForm::from(person.clone()), &csrf_tokens),
            person,
        },
        context,
    ))
}

pub(crate) async fn update_person(
    EditPersonPath { id }: EditPersonPath,
    context: Context,
    State(app_state): State<AppState>,
    DbConnection(mut conn): DbConnection,
    form: Form<PersonForm>,
) -> Result<Response, AppError> {
    let person = repository::get_person(&mut conn, &id)
        .await?
        .ok_or(person_not_found(id, context.locale))?;

    match form.validate(Some(&person), app_state.csrf_tokens()) {
        Err(form_data) => Ok(HtmlTemplate(
            PersonUpdateTemplate {
                person,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(person) => {
            repository::update_person(&mut conn, &person).await?;

            // Redirect to the address edit page
            Ok(Redirect::to(&person.edit_address_path()).into_response())
        }
    }
}
