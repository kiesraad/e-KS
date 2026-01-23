use askama::Template;
use axum::response::IntoResponse;

use crate::{Context, HtmlTemplate, filters, submit::pages::SubmitPath, t};

#[derive(Template)]
#[template(path = "submit/index.html")]
pub struct IndexTemplate {}

pub async fn index(_: SubmitPath, context: Context) -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {}, context)
}
