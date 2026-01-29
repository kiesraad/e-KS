use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, Context,
    candidate_lists::{self, CandidateList, pages::CandidateListsDeletePath},
    form::{EmptyForm, Validate},
};

pub async fn delete_candidate_list(
    _: CandidateListsDeletePath,
    context: Context,
    candidate_list: CandidateList,
    State(pool): State<PgPool>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => Ok(Redirect::to(&candidate_list.update_path()).into_response()),
        Ok(_) => {
            candidate_lists::remove_candidate_list(&pool, candidate_list.id).await?;
            Ok(Redirect::to(&CandidateList::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{StatusCode, header};
    use axum_extra::extract::Form;
    use chrono::DateTime;
    use sqlx::PgPool;

    use crate::{
        ElectoralDistrict, TokenValue,
        candidate_lists::{self, CandidateListId},
    };

    #[sqlx::test]
    async fn delete_candidate_list_and_redirect(pool: PgPool) -> Result<(), sqlx::Error> {
        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let candidate_list = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::UT],
            list_submitter_id: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        };
        candidate_lists::create_candidate_list(&pool, &candidate_list).await?;

        let response = delete_candidate_list(
            CandidateListsDeletePath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(pool.clone()),
            Form(EmptyForm { csrf_token }),
        )
        .await
        .unwrap();

        // verify redirect
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");

        assert_eq!(location, CandidateList::list_path());

        // verify deletion (i.e. no lists in database left)
        let lists = candidate_lists::list_candidate_list_summary(&pool).await?;
        assert_eq!(lists.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_candidate_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let context = Context::new_test(pool.clone()).await;
        let csrf_token = TokenValue("invalid".to_string());
        let candidate_list = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::UT],
            list_submitter_id: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        };
        candidate_lists::create_candidate_list(&pool, &candidate_list).await?;

        let response = delete_candidate_list(
            CandidateListsDeletePath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(pool.clone()),
            Form(EmptyForm { csrf_token }),
        )
        .await
        .unwrap();

        // verify redirect
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");

        assert_eq!(location, candidate_list.update_path());

        // verify deletion didn't go through (i.e. still 1 list in database left)
        let lists = candidate_lists::list_candidate_list_summary(&pool).await?;
        assert_eq!(lists.len(), 1);

        Ok(())
    }
}
