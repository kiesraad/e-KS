use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, Context, DbConnection, HtmlTemplate,
    candidate_lists::structs::{CandidateListDetail, MAX_CANDIDATES},
    filters, t,
};

use super::{CandidateList, ViewCandidateListPath, load_candidate_list};

#[derive(Template)]
#[template(path = "candidate_lists/view.html")]
struct CandidateListViewTemplate {
    details: CandidateListDetail,
    max_candidates: usize,
}

pub(crate) async fn view_candidate_list(
    ViewCandidateListPath { id }: ViewCandidateListPath,
    context: Context,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let details = load_candidate_list(&mut conn, &id, context.locale).await?;

    Ok(HtmlTemplate(
        CandidateListViewTemplate {
            details,
            max_candidates: MAX_CANDIDATES,
        },
        context,
    ))
}
