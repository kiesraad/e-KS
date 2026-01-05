use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppResponse, AppState, Context, CsrfTokens, DbConnection, HtmlTemplate, Locale,
    filters,
    form::{FormData, Validate},
    pagination::{Pagination, SortDirection},
    persons::{
        repository,
        structs::{Person, PersonForm, PersonSort},
    },
    t,
};
use uuid::Uuid;

use super::EditPersonPath;

#[derive(Template)]
#[template(path = "persons/update.html")]
struct PersonUpdateTemplate {
    person: Person,
    form: FormData<PersonForm>,
}

fn person_not_found(id: Uuid, locale: Locale) -> AppError {
    AppError::NotFound(t!("person.not_found", &locale, id))
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
