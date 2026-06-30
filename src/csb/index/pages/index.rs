use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{AppError, CsbContext, HtmlTemplate, csb::index::pages::CsbIndexPath, filters, Context};

#[derive(Template)]
#[template(path = "index/pages/index.html")]
struct CsbIndexTemplate;

pub async fn index(_: CsbIndexPath, context: CsbContext) -> Result<Response, AppError> {
    Ok(HtmlTemplate(CsbIndexTemplate, context).into_response())
}
