use crate::{
    AppError, Context, DbConnection, HtmlTemplate,
    form::{FormData, Validate},
    political_group,
    political_group::{pages::PoliticalGroupNewPath, structs::PoliticalGroupForm},
};
use askama::Template;
use axum::{
    Form,
    response::{IntoResponse, Redirect, Response},
};

#[derive(Template)]
#[template(path = "political_group/create.html")]
struct PoliticalGroupCreateTemplate {
    form: FormData<PoliticalGroupForm>,
}

pub async fn new_political_group_form(
    _: PoliticalGroupNewPath,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        PoliticalGroupCreateTemplate {
            form: FormData::new(&context.csrf_tokens),
        },
        context,
    ))
}

pub async fn create_political_group(
    _: PoliticalGroupNewPath,
    context: Context,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<PoliticalGroupForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(form) => {
            Ok(HtmlTemplate(PoliticalGroupCreateTemplate { form }, context).into_response())
        }
        Ok(political_group) => {
            political_group::create_political_group(&mut conn, &political_group).await?;

            Ok(Redirect::to("/").into_response())
        }
    }
}
