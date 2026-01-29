use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppResponse, Context, HtmlTemplate, filters,
    persons::{Person, PersonPagination, PersonSort, pages::EditPersonAuthorisedPersonPath},
};

#[derive(Template)]
#[template(path = "persons/authorised_person.html")]
struct PersonAuthorisedPersonUpdateTemplate {
    person: Person,
    person_pagination: PersonPagination,
}

pub async fn edit_authorised_person(
    _: EditPersonAuthorisedPersonPath,
    context: Context,
    person: Person,
    person_pagination: PersonPagination,
) -> AppResponse<impl IntoResponse> {
    Ok(HtmlTemplate(
        PersonAuthorisedPersonUpdateTemplate {
            person,
            person_pagination,
        },
        context,
    ))
}
