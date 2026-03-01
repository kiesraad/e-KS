use axum::response::Response;

use crate::{
    AppError, AppStore, Context, Form, candidate_lists::CandidateList, candidates::Candidate,
    form::EmptyForm, redirect_success,
};

use super::CandidateListDeletePersonPath;

pub async fn delete_person(
    _: CandidateListDeletePersonPath,
    candidate: Candidate,
    candidate_list: CandidateList,
    context: Context,
    store: AppStore,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.session.csrf_tokens) {
        Err(_) => Err(AppError::CsrfTokenInvalid),
        Ok(_) => {
            candidate.person.delete(&store).await?;

            Ok(redirect_success(candidate_list.view_path()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Form, QueryParamState,
        candidate_lists::{CandidateListId, FullCandidateList},
        persons::PersonId,
        test_utils::{sample_candidate_list, sample_person, sample_person_with_last_name},
    };
    use axum::http::{StatusCode, header};
    use axum_extra::routing::TypedPath;

    #[tokio::test]
    async fn delete_person_removes_from_list_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());
        let other_person = sample_person_with_last_name(PersonId::new(), "Other");

        person.create(&store).await?;
        other_person.create(&store).await?;
        list.candidates = vec![person.id, other_person.id];
        list.create(&store).await?;

        let candidate = store
            .get_candidate_list(list_id)?
            .get_candidate(&store, person.id)
            .await?;

        let context = Context::new_test_without_db();
        let csrf_token = context.session.csrf_tokens.issue().value;

        let response = delete_person(
            CandidateListDeletePersonPath {
                list_id,
                person_id: person.id,
            },
            candidate,
            list.clone(),
            context,
            store.clone(),
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
        assert_eq!(
            location,
            list.view_path()
                .with_query_params(QueryParamState::success())
                .to_string()
        );

        let updated_list = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(updated_list.candidates.len(), 1);
        assert_eq!(updated_list.candidates[0].person.id, other_person.id);

        let removed = store.get_person(person.id);
        assert!(removed.is_err());

        Ok(())
    }
}
