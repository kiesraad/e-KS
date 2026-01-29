use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, Context, HtmlTemplate,
    candidate_lists::{
        Candidate, CandidateList, FullCandidateList, candidate_pages::EditAuthorisedPersonPath,
    },
    filters,
};

#[derive(Template)]
#[template(path = "candidates/edit_authorised_person.html")]
struct EditAuthorisedPersonTemplate {
    full_list: FullCandidateList,
    candidate: Candidate,
}

pub async fn edit_authorised_person(
    _: EditAuthorisedPersonPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
) -> Result<impl IntoResponse, AppError> {
    Ok(HtmlTemplate(
        EditAuthorisedPersonTemplate {
            candidate: candidate.clone(),
            full_list,
        },
        context,
    ))
}
