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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};
    use axum_extra::extract::Form;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        Context, CsrfTokens, DbConnection, Locale, TokenValue, candidate_lists, persons,
        test_utils::{
            response_body_string, sample_candidate_list, sample_person,
            sample_person_with_last_name,
        },
    };

    fn sample_position_form(
        csrf_token: &TokenValue,
        position: usize,
        action: &str,
    ) -> PositionForm {
        PositionForm {
            position: position.to_string(),
            action: action.to_string(),
            csrf_token: csrf_token.clone(),
        }
    }

    #[sqlx::test]
    async fn edit_candidate_position_renders_form(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person = sample_person(Uuid::new_v4());

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;
        candidate_lists::repository::update_candidate_list_order(&mut conn, &list_id, &[person.id])
            .await?;

        let response = edit_candidate_position(
            EditCandidatePositionPath {
                candidate_list: list_id,
                person: person.id,
            },
            Context::new(Locale::En),
            CsrfTokens::default(),
            DbConnection(pool.acquire().await?),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list.edit_candidate_position_path(&person.id)));
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_position_moves_candidate(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(Uuid::new_v4(), "Jansen");
        let person_b = sample_person_with_last_name(Uuid::new_v4(), "Bakker");

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person_a).await?;
        persons::repository::create_person(&mut conn, &person_b).await?;
        candidate_lists::repository::update_candidate_list_order(
            &mut conn,
            &list_id,
            &[person_a.id, person_b.id],
        )
        .await?;

        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let form = sample_position_form(&csrf_token, 2, "move");

        let response = update_candidate_position(
            EditCandidatePositionPath {
                candidate_list: list_id,
                person: person_a.id,
            },
            Context::new(Locale::En),
            csrf_tokens,
            DbConnection(pool.acquire().await?),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let mut conn = pool.acquire().await?;
        let full_list = super::super::load_candidate_list(&mut conn, &list_id, Locale::En)
            .await
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, person_b.id);
        assert_eq!(full_list.candidates[1].person.id, person_a.id);

        Ok(())
    }
}
