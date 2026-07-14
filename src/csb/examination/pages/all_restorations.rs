use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, Context, CsbContext, CsbStore, HtmlTemplate,
    csb::examination::{extractors::CsbPoliticalGroup, pages::CsbAllRestorationsPath},
    filters,
};

#[derive(Template)]
#[template(path = "csb/examination/pages/all_restorations.html")]
struct CsbAllRestorationsTemplate {
    political_group: CsbPoliticalGroup,
    restoration_count: usize,
}

pub async fn all_restorations(
    _: CsbAllRestorationsPath,
    context: CsbContext,
    store: CsbStore,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    Ok(HtmlTemplate(
        CsbAllRestorationsTemplate {
            political_group,
            restoration_count: 4,
        },
        context,
    )
    .into_response())
}
