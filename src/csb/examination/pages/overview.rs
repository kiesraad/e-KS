use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, Context, CsbContext, HtmlTemplate, csb::{
        examination::pages::CsbExaminationOverviewPath, import::{CsbImportPath, CsbPoliticalGroup, CsbPoliticalGroups},
    }, filters,
};

#[derive(Template)]
#[template(path = "examination/pages/overview.html")]
struct CsbExaminationOverviewTemplate {
    political_groups: Vec<CsbPoliticalGroup>,
}

/// Render the placeholder overview page.
pub async fn overview(
    _: CsbExaminationOverviewPath,
    context: CsbContext,
    CsbPoliticalGroups(political_groups): CsbPoliticalGroups,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(CsbExaminationOverviewTemplate { political_groups }, context).into_response())
}
