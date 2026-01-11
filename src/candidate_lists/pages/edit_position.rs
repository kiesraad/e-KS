use askama::Template;
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::Form;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError, Context, CsrfToken, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        pages::{EditCandidatePositionPath, load_candidate_list},
        repository,
        structs::{CandidateList, CandidateListDetail, CandidateListEntry},
    },
    filters,
    persons::structs::Person,
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/edit_position.html")]
struct EditCandidatePositionTemplate {
    details: CandidateListDetail,
    candidate: CandidateListEntry,
    csrf_token: CsrfToken,
    max_candidates: usize,
}

pub async fn edit_candidate_position(
    EditCandidatePositionPath {
        candidate_list,
        person,
    }: EditCandidatePositionPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let details: CandidateListDetail =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;

    let candidate = details
        .candidates
        .iter()
        .find(|c| c.person.id == person)
        .ok_or_else(|| AppError::NotFound("Person not found in candidate list".to_string()))?;

    // Implementation for editing candidate position goes here
    Ok(HtmlTemplate(
        EditCandidatePositionTemplate {
            candidate: candidate.clone(),
            details,
            csrf_token: csrf_tokens.issue(),
            max_candidates: 50, // TODO: determine max_candidates from political group configuration
        },
        context,
    ))
}

#[derive(Deserialize)]
pub struct UpdateCandidatePositionForm {
    pub new_position: usize,
    pub action: String,
    pub csrf_token: String,
}

pub async fn update_candidate_position(
    EditCandidatePositionPath {
        candidate_list,
        person,
    }: EditCandidatePositionPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<UpdateCandidatePositionForm>,
) -> Result<impl IntoResponse, AppError> {
    let details: CandidateListDetail =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;
    let mut person_ids: Vec<Uuid> = details.candidates.iter().map(|c| c.person.id).collect();

    let Some(current_index) = details.index(&person) else {
        return Err(AppError::NotFound(
            "Person not found in candidate list".to_string(),
        ));
    };

    // redirect back to the form if the CSRF token is invalid
    if !csrf_tokens.consume(&form.csrf_token) {
        return Ok(Redirect::to(
            &details.list.edit_candidate_position_path(&person),
        ));
    }

    let moved = person_ids.remove(current_index);

    if form.action == "remove" {
        repository::update_candidate_list(&mut conn, &candidate_list, &person_ids).await?;
    } else if form.action == "move" {
        let target_index = form
            .new_position
            .saturating_sub(1)
            .min(person_ids.len() - 1);

        if current_index != target_index {
            person_ids.insert(target_index, moved);
            repository::update_candidate_list(&mut conn, &candidate_list, &person_ids).await?;
        }
    }

    Ok(Redirect::to(&details.list.view_path()))
}
