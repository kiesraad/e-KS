use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection,
    candidate_lists::{
        self, CandidateList,
        candidate_pages::{CandidateListDeletePersonPath, CandidateListEditPersonPath},
    },
    form::{EmptyForm, Validate},
    persons,
};

pub async fn delete_person(
    CandidateListDeletePersonPath {
        candidate_list,
        person,
    }: CandidateListDeletePersonPath,
    context: Context,
    _: State<AppState>,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    form: Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate(None, &csrf_tokens) {
        Err(_) => {
            // csrf token is invalid => back to edit view
            Ok(Redirect::to(
                &CandidateListEditPersonPath {
                    candidate_list,
                    person,
                }
                .to_string(),
            )
            .into_response())
        }
        Ok(_) => {
            let full_list = candidate_lists::pages::load_candidate_list(
                &mut conn,
                candidate_list,
                context.locale,
            )
            .await?;
            let candidate = full_list.get_candidate(&person, context.locale)?;

            // remove person from list
            let mut updates_ids = full_list.get_ids();
            updates_ids.retain(|id| id != &candidate.person.id);
            candidate_lists::repository::update_candidate_list_order(
                &mut conn,
                candidate_list,
                &updates_ids,
            )
            .await?;

            persons::repository::remove_person(&mut conn, &candidate.person.id).await?;

            Ok(Redirect::to(&CandidateList::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::State,
        http::{StatusCode, header},
    };
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        AppState, Context, CsrfTokens, DbConnection, Locale,
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
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;
        persons::repository::create_person(&mut conn, &other_person).await?;
        candidate_lists::repository::update_candidate_list_order(
            &mut conn,
            list_id,
            &[person.id, other_person.id],
        )
        .await?;

        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;

        let response = delete_person(
            CandidateListDeletePersonPath {
                candidate_list: list_id,
                person: person.id,
            },
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens.clone(),
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
        let updated_list = candidate_lists::repository::get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(updated_list.candidates.len(), 1);
        assert_eq!(updated_list.candidates[0].person.id, other_person.id);

        let removed = persons::repository::get_person(&mut conn, &person.id).await?;
        assert!(removed.is_none());

        Ok(())
    }
}
