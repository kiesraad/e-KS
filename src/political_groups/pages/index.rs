use askama::Template;
use axum::response::IntoResponse;

use crate::{Context, HtmlTemplate, filters, political_groups::pages::PoliticalGroupPath, t};

#[derive(Template)]
#[template(path = "political_groups/index.html")]
pub struct IndexTemplate {}

pub async fn index(_: PoliticalGroupPath, context: Context) -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {}, context)
}
