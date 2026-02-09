use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, candidate_lists::CandidateList, candidates::Candidate,
    form::EmptyForm,
};

use super::CandidateListDeletePersonPath;
pub async fn delete_person(
    _: CandidateListDeletePersonPath,
    candidate: Candidate,
    candidate_list: CandidateList,
    context: Context,
    State(store): State<AppStore>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => {
            // TODO: set error flash message
            Ok(Redirect::to(&candidate.update_path()).into_response())
        }
        Ok(_) => {
            candidate.person.delete(&store).await?;
            // TODO: set success flash message
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppEvent, AppStore,
        candidate_lists::{CandidateListId, FullCandidateList},
        persons::PersonId,
        test_utils::{sample_candidate_list, sample_person, sample_person_with_last_name},
    };
    use axum::http::{StatusCode, header};
    use axum_extra::extract::Form;

    #[tokio::test]
    async fn delete_person_removes_from_list_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());
        let other_person = sample_person_with_last_name(PersonId::new(), "Other");

        list.create(&store).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;
        store
            .update(AppEvent::CreatePerson(other_person.clone()))
            .await?;
        CandidateList::update_order(&store, list_id, &[person.id, other_person.id]).await?;
        let candidate = CandidateList::get_candidate(&store, list_id, person.id).await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_person(
            CandidateListDeletePersonPath {
                list_id,
                person_id: person.id,
            },
            candidate,
            list.clone(),
            context,
            State(store.clone()),
            Form(EmptyForm::new(csrf_token)),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(location, list.view_path());

        let updated_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(updated_list.candidates.len(), 1);
        assert_eq!(updated_list.candidates[0].person.id, other_person.id);

        let removed = store.get_person(person.id);
        assert!(removed.is_err());

        Ok(())
    }
}
