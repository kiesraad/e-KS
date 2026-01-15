use askama::Template;
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::Form;
use uuid::Uuid;

use crate::{
    AppError, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        pages::{EditCandidatePositionPath, load_candidate_list},
        repository,
        structs::{
            CandidateList, CandidateListDetail, CandidateListEntry, CandidatePosition,
            CandidatePositionAction, MAX_CANDIDATES, PositionForm,
        },
    },
    filters,
    form::{FormData, Validate},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/edit_position.html")]
struct EditCandidatePositionTemplate {
    details: CandidateListDetail,
    candidate: CandidateListEntry,
    form: FormData<PositionForm>,
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

    let candidate_position = CandidatePosition {
        position: candidate.position as usize,
        action: CandidatePositionAction::Move,
    };

    let form =
        FormData::new_with_data(PositionForm::from(candidate_position.clone()), &csrf_tokens);

    // Implementation for editing candidate position goes here
    Ok(HtmlTemplate(
        EditCandidatePositionTemplate {
            candidate: candidate.clone(),
            details,
            form,
            max_candidates: MAX_CANDIDATES,
        },
        context,
    ))
}

pub async fn update_candidate_position(
    EditCandidatePositionPath {
        candidate_list,
        person,
    }: EditCandidatePositionPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<PositionForm>,
) -> Result<impl IntoResponse, AppError> {
    let details: CandidateListDetail =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;
    let mut person_ids: Vec<Uuid> = details.candidates.iter().map(|c| c.person.id).collect();

    let Some(current_index) = details.index(&person) else {
        return Err(AppError::NotFound(
            "Person not found in candidate list".to_string(),
        ));
    };

    let candidate = &details.candidates[current_index];

    let candidate_position = CandidatePosition {
        position: candidate.position as usize,
        action: CandidatePositionAction::Move,
    };

    match form.validate(Some(&candidate_position), &csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            EditCandidatePositionTemplate {
                candidate: candidate.clone(),
                details,
                form: form_data,
                max_candidates: MAX_CANDIDATES,
            },
            context,
        )
        .into_response()),
        Ok(position_form) => {
            let moved = person_ids.remove(current_index);

            if position_form.action == CandidatePositionAction::Remove {
                repository::update_candidate_list_order(&mut conn, &candidate_list, &person_ids)
                    .await?;
            } else if position_form.action == CandidatePositionAction::Move {
                let target_index = position_form
                    .position
                    .saturating_sub(1)
                    .min(person_ids.len());

                if current_index != target_index {
                    person_ids.insert(target_index, moved);
                    repository::update_candidate_list_order(
                        &mut conn,
                        &candidate_list,
                        &person_ids,
                    )
                    .await?;
                }
            }

            Ok(Redirect::to(&details.list.view_path()).into_response())
        }
    }
}
