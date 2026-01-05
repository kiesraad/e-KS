use axum::response::{IntoResponse, Redirect, Response};

use crate::{
    AppError, DbConnection,
    persons::{repository, structs::Person},
};

use super::DeletePersonPath;

pub(crate) async fn delete_person(
    DeletePersonPath { id }: DeletePersonPath,
    DbConnection(mut conn): DbConnection,
) -> Result<Response, AppError> {
    repository::remove_person(&mut conn, &id).await?;

    Ok(Redirect::to(&Person::list_path()).into_response())
}
