use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, AppStore, Context, Form, HtmlTemplate,
    candidate_lists::FullCandidateList,
    candidates::{Candidate, CandidatePosition, CandidatePositionForm},
    common::FormAction,
    filters,
    form::FormData,
    redirect_success,
};

use super::UpdateCandidatePositionPath;

#[derive(Template)]
#[template(path = "candidates/pages/update_position.html")]
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
        action: FormAction::Save,
    };

    let form = FormData::new_with_data(
        CandidatePositionForm::from(candidate_position.clone()),
        &context.session.csrf_tokens,
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
    store: AppStore,
    Form(form): Form<CandidatePositionForm>,
) -> Result<impl IntoResponse, AppError> {
    let candidate_position = CandidatePosition {
        position: candidate.position,
        action: FormAction::Save,
    };

    match form.validate_update(&candidate_position, &context.session.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            UpdateCandidatePositionTemplate {
                candidate,
                full_list,
                form: form_data,
            },
            context,
        )
        .into_response()),
        Ok(position_form) => match position_form.action {
            FormAction::Remove => {
                let mut list = full_list.list;
                list.remove_candidate(&store, candidate.person.id).await?;

                Ok(redirect_success(list.view_path()))
            }
            FormAction::Save => {
                let mut list = full_list.list;
                list.update_position(&store, candidate.person.id, position_form.position)
                    .await?;

                Ok(redirect_success(candidate.update_path()))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context, Form, TokenValue,
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{
            response_body_string, sample_candidate_list, sample_person,
            sample_person_with_last_name,
        },
    };
    use axum::{http::StatusCode, response::IntoResponse};

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
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        person.create(&store).await?;
        list.candidates = vec![person.id];
        list.create(&store).await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = list.get_candidate(&store, person.id).await?;

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
        assert!(body.contains(&candidate.update_position_path().to_string()));
        assert!(body.contains("Jansen"));

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_position_moves_candidate() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        person_a.create(&store).await?;
        person_b.create(&store).await?;
        list.clone()
            .update_order(&store, &[person_a.id, person_b.id])
            .await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = store
            .get_candidate_list(list_id)?
            .get_candidate(&store, person_a.id)
            .await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let form = sample_position_form(&csrf_token, 2, "save");

        let response = update_candidate_position_submit(
            UpdateCandidatePositionPath {
                list_id,
                person_id: person_a.id,
            },
            context,
            full_list,
            candidate,
            store.clone(),
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
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        person_a.create(&store).await?;
        person_b.create(&store).await?;
        list.clone()
            .update_order(&store, &[person_a.id, person_b.id])
            .await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = store
            .get_candidate_list(list_id)?
            .get_candidate(&store, person_a.id)
            .await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;
        let form = sample_position_form(&csrf_token, 1, "remove");

        let response = update_candidate_position_submit(
            UpdateCandidatePositionPath {
                list_id,
                person_id: person_a.id,
            },
            context,
            full_list,
            candidate,
            store.clone(),
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
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        person_a.create(&store).await?;
        person_b.create(&store).await?;
        list.clone()
            .update_order(&store, &[person_a.id, person_b.id])
            .await?;

        let full_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        let candidate = store
            .get_candidate_list(list_id)?
            .get_candidate(&store, person_a.id)
            .await?;

        let context = Context::new_test_without_db();
        let csrf_token = TokenValue("invalid".to_string());
        let form = sample_position_form(&csrf_token, 2, "save");

        let response = update_candidate_position_submit(
            UpdateCandidatePositionPath {
                list_id,
                person_id: person_a.id,
            },
            context,
            full_list,
            candidate,
            store.clone(),
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
