use askama::Template;
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::Form;

use crate::{
    AppError, Context, DbConnection, HtmlTemplate,
    candidate_lists::{
        self, Candidate, CandidateList, CandidatePosition, CandidatePositionAction,
        CandidatePositionForm, FullCandidateList, candidate_pages::EditCandidatePositionPath,
    },
    filters,
    form::{FormData, Validate},
};

#[derive(Template)]
#[template(path = "candidates/edit_position.html")]
struct EditCandidatePositionTemplate {
    full_list: FullCandidateList,
    candidate: Candidate,
    form: FormData<CandidatePositionForm>,
}

pub async fn edit_candidate_position(
    _: EditCandidatePositionPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
) -> Result<impl IntoResponse, AppError> {
    let candidate_position = CandidatePosition {
        position: candidate.position,
        action: CandidatePositionAction::Move,
    };

    let form = FormData::new_with_data(
        CandidatePositionForm::from(candidate_position.clone()),
        &context.csrf_tokens,
    );

    // Implementation for editing candidate position goes here
    Ok(HtmlTemplate(
        EditCandidatePositionTemplate {
            candidate: candidate.clone(),
            full_list,
            form,
        },
        context,
    ))
}

pub async fn update_candidate_position(
    _: EditCandidatePositionPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<CandidatePositionForm>,
) -> Result<impl IntoResponse, AppError> {
    let candidate_position = CandidatePosition {
        position: candidate.position,
        action: CandidatePositionAction::Move,
    };

    match form.validate_update(&candidate_position, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            EditCandidatePositionTemplate {
                candidate,
                full_list,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(position_form) => {
            let redirect = Redirect::to(&full_list.list.view_path()).into_response();

            match position_form.action {
                CandidatePositionAction::Remove => {
                    candidate_lists::remove_candidate(
                        &mut conn,
                        candidate.list_id,
                        candidate.person.id,
                    )
                    .await?;
                }
                CandidatePositionAction::Move => {
                    let mut full_list = full_list;
                    full_list.update_position(candidate.person.id, position_form.position);
                    candidate_lists::update_candidate_list_order(
                        &mut conn,
                        candidate.list_id,
                        &full_list.get_ids(),
                    )
                    .await?;
                }
            }

            Ok(redirect)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        Context, DbConnection, TokenValue,
        candidate_lists::{self, CandidateListId},
        persons::{self, PersonId},
        test_utils::{
            response_body_string, sample_candidate_list, sample_person,
            sample_person_with_last_name,
        },
    };

    fn sample_position_form(
        csrf_token: &TokenValue,
        position: usize,
        action: &str,
    ) -> CandidatePositionForm {
        CandidatePositionForm {
            position: position.to_string(),
            action: action.to_string(),
            csrf_token: csrf_token.clone(),
        }
    }

    #[sqlx::test]
    async fn edit_candidate_position_renders_form(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person).await?;
        candidate_lists::update_candidate_list_order(&mut conn, list_id, &[person.id]).await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&mut conn, list_id, person.id).await?;

        let response = edit_candidate_position(
            EditCandidatePositionPath {
                list_id,
                person_id: person.id,
            },
            Context::new_test(),
            full_list,
            candidate.clone(),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&candidate.edit_position_path()));
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_position_moves_candidate(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person_a).await?;
        persons::create_person(&mut conn, &person_b).await?;
        candidate_lists::update_candidate_list_order(
            &mut conn,
            list_id,
            &[person_a.id, person_b.id],
        )
        .await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&mut conn, list_id, person_a.id).await?;

        let context = Context::new_test();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_position_form(&csrf_token, 2, "move");

        let response = update_candidate_position(
            EditCandidatePositionPath {
                list_id,
                person_id: person_a.id,
            },
            context,
            full_list,
            candidate,
            DbConnection(pool.acquire().await?),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let mut conn = pool.acquire().await?;
        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, person_b.id);
        assert_eq!(full_list.candidates[1].person.id, person_a.id);

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_position_removes_candidate(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person_a).await?;
        persons::create_person(&mut conn, &person_b).await?;
        candidate_lists::update_candidate_list_order(
            &mut conn,
            list_id,
            &[person_a.id, person_b.id],
        )
        .await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&mut conn, list_id, person_a.id).await?;

        let context = Context::new_test();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_position_form(&csrf_token, 1, "remove");

        let response = update_candidate_position(
            EditCandidatePositionPath {
                list_id,
                person_id: person_a.id,
            },
            context,
            full_list,
            candidate,
            DbConnection(pool.acquire().await?),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let mut conn = pool.acquire().await?;
        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 1);
        assert_eq!(full_list.candidates[0].person.id, person_b.id);

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_position_invalid_csrf_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person_a).await?;
        persons::create_person(&mut conn, &person_b).await?;
        candidate_lists::update_candidate_list_order(
            &mut conn,
            list_id,
            &[person_a.id, person_b.id],
        )
        .await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        let candidate = candidate_lists::get_candidate(&mut conn, list_id, person_a.id).await?;

        let context = Context::new_test();
        let csrf_token = TokenValue("invalid".to_string());
        let form = sample_position_form(&csrf_token, 2, "move");

        let response = update_candidate_position(
            EditCandidatePositionPath {
                list_id,
                person_id: person_a.id,
            },
            context,
            full_list,
            candidate,
            DbConnection(pool.acquire().await?),
            Form(form),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("The CSRF token is invalid."));

        let mut conn = pool.acquire().await?;
        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, person_a.id);
        assert_eq!(full_list.candidates[1].person.id, person_b.id);

        Ok(())
    }
}
