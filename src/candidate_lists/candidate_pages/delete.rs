use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, Context,
    candidate_lists::{
        self, Candidate, CandidateList, candidate_pages::CandidateListDeletePersonPath,
    },
    form::{EmptyForm, Validate},
    persons,
};

pub async fn delete_person(
    _: CandidateListDeletePersonPath,
    candidate: Candidate,
    candidate_list: CandidateList,
    context: Context,
    State(pool): State<PgPool>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => {
            // TODO: set error flash message
            Ok(Redirect::to(&candidate.edit_path()).into_response())
        }
        Ok(_) => {
            candidate_lists::remove_candidate(&pool, candidate.person.id).await?;

            persons::remove_person(&pool, candidate.person.id).await?;
            // TODO: set success flash message
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{StatusCode, header};
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        candidate_lists::{self, CandidateListId},
        persons::{self, PersonId},
        test_utils::{sample_candidate_list, sample_person, sample_person_with_last_name},
    };

    #[sqlx::test]
    async fn delete_person_removes_from_list_and_redirects(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());
        let other_person = sample_person_with_last_name(PersonId::new(), "Other");

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person).await?;
        persons::create_person(&pool, &other_person).await?;
        candidate_lists::update_candidate_list_order(&pool, list_id, &[person.id, other_person.id])
            .await?;
        let candidate = candidate_lists::get_candidate(&pool, list_id, person.id).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_person(
            CandidateListDeletePersonPath {
                list_id,
                person_id: person.id,
            },
            candidate,
            list.clone(),
            context,
            State(pool.clone()),
            Form(EmptyForm::from(csrf_token)),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(location, list.view_path());

        let updated_list = candidate_lists::get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(updated_list.candidates.len(), 1);
        assert_eq!(updated_list.candidates[0].person.id, other_person.id);

        let removed = persons::get_person(&pool, person.id).await?;
        assert!(removed.is_none());

        Ok(())
    }
}
