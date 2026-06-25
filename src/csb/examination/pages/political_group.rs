use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{AppError, Context, CsbContext, CsbStore, HtmlTemplate, csb::examination::pages::CsbPoliticalGroupPath, filters, political_groups::PoliticalGroup};

#[derive(Template)]
#[template(path = "examination/pages/political_group.html")]
struct CsbPoliticalGroupTemplate {
    political_group_name: String
}

/// Render the placeholder political group overview page.
pub async fn overview(
    _: CsbPoliticalGroupPath,
    context: CsbContext,
    store: CsbStore
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(CsbPoliticalGroupTemplate {
        political_group_name: store.data.read().imported_data.political_group.display_name.as_ref().map_or("?".to_string(), |dn| dn.to_string())
    }, context).into_response())
}
