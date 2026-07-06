use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, Context, CsbContext, HtmlTemplate, csb::index::pages::CsbIndexPath, filters,
};

#[derive(Template)]
#[template(path = "csb/index/pages/index.html")]
struct CsbIndexTemplate;

pub async fn index(_: CsbIndexPath, context: CsbContext) -> Result<Response, AppError> {
    Ok(HtmlTemplate(CsbIndexTemplate, context).into_response())
}
