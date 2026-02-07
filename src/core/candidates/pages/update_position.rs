use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    candidate_lists::{CandidateList, FullCandidateList},
    candidates::{Candidate, CandidatePosition, CandidatePositionAction, CandidatePositionForm},
    filters,
    form::FormData,
};

use super::UpdateCandidatePositionPath;

#[derive(Template)]
#[template(path = "candidates/update_position.html")]
struct UpdateCandidatePositionTemplate {
    full_list: FullCandidateList,
    candidate: Candidate,
    form: FormData<CandidatePositionForm>,
}

pub async fn update_candidate_position(
    _: UpdateCandidatePositionPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
) -> Result<impl IntoResponse, AppError> {
    let candidate_position = CandidatePosition {
        position: candidate.position,
        action: CandidatePositionAction::Move,
        ..Default::default()
    };

    let form = FormData::new_with_data(
        CandidatePositionForm::from(candidate_position.clone()),
        &context.csrf_tokens,
    );

    // Implementation for editing candidate position goes here
    Ok(HtmlTemplate(
        UpdateCandidatePositionTemplate {
            candidate: candidate.clone(),
            full_list,
            form,
        },
        context,
    ))
}

pub async fn update_candidate_position_submit(
    _: UpdateCandidatePositionPath,
    context: Context,
    full_list: FullCandidateList,
    candidate: Candidate,
    State(store): State<AppStore>,
    Form(form): Form<CandidatePositionForm>,
) -> Result<impl IntoResponse, AppError> {
    let candidate_position = CandidatePosition {
        position: candidate.position,
        action: CandidatePositionAction::Move,
        ..Default::default()
    };

    match form.validate_update(&candidate_position, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            UpdateCandidatePositionTemplate {
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
                    CandidateList::remove_candidate_from_list(
                        &store,
                        candidate.list_id,
                        candidate.person.id,
                    )
                    .await?;
                }
                CandidatePositionAction::Move => {
                    let mut full_list = full_list;
                    full_list.update_position(candidate.person.id, position_form.position);
                    CandidateList::update_order(&store, candidate.list_id, &full_list.get_ids())
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
    use crate::{
        AppEvent, AppStore, Context, TokenValue,
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{
            response_body_string, sample_candidate_list, sample_person,
            sample_person_with_last_name,
        },
    };
    use axum::{http::StatusCode, response::IntoResponse};
    use axum_extra::extract::Form;

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

    #[tokio::test]
    async fn update_candidate_position_renders_form() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;
        CandidateList::update_order(&store, list_id, &[person.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = CandidateList::get_candidate(&store, list_id, person.id).await?;

        let response = update_candidate_position(
            UpdateCandidatePositionPath {
                list_id,
                person_id: person.id,
            },
            Context::new_test_without_db(),
            full_list,
            candidate.clone(),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&candidate.update_position_path()));
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_position_moves_candidate() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        store
            .update(AppEvent::CreatePerson(person_a.clone()))
            .await?;
        store
            .update(AppEvent::CreatePerson(person_b.clone()))
            .await?;
        CandidateList::update_order(&store, list_id, &[person_a.id, person_b.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = CandidateList::get_candidate(&store, list_id, person_a.id).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_position_form(&csrf_token, 2, "move");

        let response = update_candidate_position_submit(
            UpdateCandidatePositionPath {
                list_id,
                person_id: person_a.id,
            },
            context,
            full_list,
            candidate,
            State(store.clone()),
            Form(form),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, person_b.id);
        assert_eq!(full_list.candidates[1].person.id, person_a.id);

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_position_removes_candidate() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        store
            .update(AppEvent::CreatePerson(person_a.clone()))
            .await?;
        store
            .update(AppEvent::CreatePerson(person_b.clone()))
            .await?;
        CandidateList::update_order(&store, list_id, &[person_a.id, person_b.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = CandidateList::get_candidate(&store, list_id, person_a.id).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let form = sample_position_form(&csrf_token, 1, "remove");

        let response = update_candidate_position_submit(
            UpdateCandidatePositionPath {
                list_id,
                person_id: person_a.id,
            },
            context,
            full_list,
            candidate,
            State(store.clone()),
            Form(form),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(full_list.candidates.len(), 1);
        assert_eq!(full_list.candidates[0].person.id, person_b.id);

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_position_invalid_csrf_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        store
            .update(AppEvent::CreatePerson(person_a.clone()))
            .await?;
        store
            .update(AppEvent::CreatePerson(person_b.clone()))
            .await?;
        CandidateList::update_order(&store, list_id, &[person_a.id, person_b.id]).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = CandidateList::get_candidate(&store, list_id, person_a.id).await?;

        let context = Context::new_test_without_db();
        let csrf_token = TokenValue("invalid".to_string());
        let form = sample_position_form(&csrf_token, 2, "move");

        let response = update_candidate_position_submit(
            UpdateCandidatePositionPath {
                list_id,
                person_id: person_a.id,
            },
            context,
            full_list,
            candidate,
            State(store.clone()),
            Form(form),
        )
        .await?
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("The CSRF token is invalid."));

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(full_list.candidates.len(), 2);
        assert_eq!(full_list.candidates[0].person.id, person_a.id);
        assert_eq!(full_list.candidates[1].person.id, person_b.id);

        Ok(())
    }
}
