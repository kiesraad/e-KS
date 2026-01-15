use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, Context, DbConnection, HtmlTemplate,
    candidate_lists::structs::{FullCandidateList, MAX_CANDIDATES},
    filters, t,
};

use super::{CandidateList, ViewCandidateListPath, load_candidate_list};

#[derive(Template)]
#[template(path = "candidate_lists/view.html")]
struct CandidateListViewTemplate {
    full_list: FullCandidateList,
    max_candidates: usize,
}

pub(crate) async fn view_candidate_list(
    ViewCandidateListPath { id }: ViewCandidateListPath,
    context: Context,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let full_list = load_candidate_list(&mut conn, &id, context.locale).await?;

    Ok(HtmlTemplate(
        CandidateListViewTemplate {
            full_list,
            max_candidates: MAX_CANDIDATES,
        },
        context,
    ))
}
