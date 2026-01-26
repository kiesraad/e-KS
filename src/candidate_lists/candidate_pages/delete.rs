use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::{
    AppError, CsrfTokens, DbConnection,
    candidate_lists::{
        self, Candidate, CandidateList, candidate_pages::CandidateListDeletePersonPath,
    },
    form::{EmptyForm, Validate},
    persons,
};

pub async fn delete_person(
    _: CandidateListDeletePersonPath,
    csrf_tokens: CsrfTokens,
    candidate: Candidate,
    DbConnection(mut conn): DbConnection,
    form: Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&csrf_tokens) {
        Err(_) => Ok(Redirect::to(&candidate.edit_path()).into_response()),
        Ok(_) => {
            candidate_lists::remove_candidate(&mut conn, candidate.list_id, candidate.person.id)
                .await?;

            persons::remove_person(&mut conn, candidate.person.id).await?;

            Ok(Redirect::to(&CandidateList::list_path()).into_response())
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
        CsrfTokens, DbConnection,
        candidate_lists::{self, CandidateList, CandidateListId},
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

        let mut conn = pool.acquire().await?;
        candidate_lists::create_candidate_list(&mut conn, &list).await?;
        persons::create_person(&mut conn, &person).await?;
        persons::create_person(&mut conn, &other_person).await?;
        candidate_lists::update_candidate_list_order(
            &mut conn,
            list_id,
            &[person.id, other_person.id],
        )
        .await?;
        let candidate = candidate_lists::get_candidate(&mut conn, list_id, person.id).await?;

        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;

        let response = delete_person(
            CandidateListDeletePersonPath {
                list_id,
                person_id: person.id,
            },
            csrf_tokens,
            candidate,
            DbConnection(pool.acquire().await?),
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
        assert_eq!(location, CandidateList::list_path());

        let mut conn = pool.acquire().await?;
        let updated_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(updated_list.candidates.len(), 1);
        assert_eq!(updated_list.candidates[0].person.id, other_person.id);

        let removed = persons::get_person(&mut conn, person.id).await?;
        assert!(removed.is_none());

        Ok(())
    }
}
