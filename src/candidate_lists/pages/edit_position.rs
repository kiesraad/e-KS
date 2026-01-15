use askama::Template;
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::Form;

use crate::{
    AppError, Context, CsrfTokens, DbConnection, HtmlTemplate,
    candidate_lists::{
        self,
        pages::{EditCandidatePositionPath, load_candidate_list},
        structs::{
            CandidateList, CandidateListEntry, CandidatePosition, CandidatePositionAction,
            FullCandidateList, MAX_CANDIDATES, PositionForm,
        },
    },
    filters,
    form::{FormData, Validate},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/edit_position.html")]
struct EditCandidatePositionTemplate {
    full_list: FullCandidateList,
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
    let full_list: FullCandidateList =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;
    let candidate = full_list.get_candidate(&person, context.locale)?;

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
            full_list,
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
    let full_list: FullCandidateList =
        load_candidate_list(&mut conn, &candidate_list, context.locale).await?;
    let mut person_ids = full_list.get_ids();

    let Some(current_index) = full_list.get_index(&person) else {
        return Err(AppError::NotFound(
            "Person not found in candidate list".to_string(),
        ));
    };

    let candidate = full_list.get_candidate(&person, context.locale)?;

    let candidate_position = CandidatePosition {
        position: candidate.position as usize,
        action: CandidatePositionAction::Move,
    };

    match form.validate(Some(&candidate_position), &csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            EditCandidatePositionTemplate {
                candidate,
                full_list,
                form: form_data,
                max_candidates: MAX_CANDIDATES,
            },
            context,
        )
        .into_response()),
        Ok(position_form) => {
            let moved = person_ids.remove(current_index);

            if position_form.action == CandidatePositionAction::Remove {
                candidate_lists::repository::update_candidate_list_order(
                    &mut conn,
                    &candidate_list,
                    &person_ids,
                )
                .await?;
            } else if position_form.action == CandidatePositionAction::Move {
                let target_index = position_form
                    .position
                    .saturating_sub(1)
                    .min(person_ids.len());

                if current_index != target_index {
                    person_ids.insert(target_index, moved);
                    candidate_lists::repository::update_candidate_list_order(
                        &mut conn,
                        &candidate_list,
                        &person_ids,
                    )
                    .await?;
                }
            }

            Ok(Redirect::to(&full_list.list.view_path()).into_response())
        }
    }
}
