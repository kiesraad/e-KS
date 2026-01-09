use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate, filters,
    form::{FormData, Validate},
    persons::{
        repository,
        structs::{Person, PersonForm},
    },
    t,
};

use super::PersonsNewPath;

#[derive(Template)]
#[template(path = "persons/create.html")]
struct PersonCreateTemplate {
    form: FormData<PersonForm>,
}

pub(crate) async fn new_person_form(
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

pub(crate) async fn create_person(
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
            repository::create_person(&mut conn, &person).await?;

            // Redirect to the address edit page
            Ok(Redirect::to(&person.edit_address_path()).into_response())
        }
    }
}
