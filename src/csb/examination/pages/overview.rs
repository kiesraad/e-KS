use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{AppError, CsbContext, Context, HtmlTemplate, csb::examination::pages::CsbExaminationOverviewPath, filters};

#[derive(Template)]
#[template(path = "examination/pages/overview.html")]
struct CsbExaminationOverviewTemplate {
}

/// Render the placeholder overview page.
pub async fn overview(_: CsbExaminationOverviewPath, context: CsbContext) -> Result<Response, AppError> {
    Ok(HtmlTemplate(CsbExaminationOverviewTemplate {}, context).into_response())
}
