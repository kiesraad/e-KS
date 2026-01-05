use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, Context, DbConnection, HtmlTemplate, filters,
    pagination::{Pagination, PaginationInfo},
    persons::{
        repository,
        structs::{Person, PersonSort},
    },
    t,
};

use super::PersonsPath;

#[derive(Template)]
#[template(path = "persons/list.html")]
struct PersonListTemplate {
    persons: Vec<Person>,
    pagination: PaginationInfo<PersonSort>,
}

pub(crate) async fn list_persons(
    _: PersonsPath,
    context: Context,
    pagination: Pagination<PersonSort>,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let total_items = repository::count_persons(&mut conn).await?.max(0) as u64;
    let pagination = pagination.set_total(total_items);

    let persons = repository::list_persons(
        &mut conn,
        pagination.limit(),
        pagination.offset(),
        pagination.sort(),
        pagination.direction(),
    )
    .await?;

    Ok(HtmlTemplate(
        PersonListTemplate {
            persons,
            pagination,
        },
        context,
    ))
}
