use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::{
    AppError, CsrfTokens, DbConnection,
    candidate_lists::{self, CandidateList, pages::CandidateListsDeletePath},
    form::{EmptyForm, Validate},
};

pub async fn delete_candidate_list(
    CandidateListsDeletePath { .. }: CandidateListsDeletePath,
    csrf_tokens: CsrfTokens,
    candidate_list: CandidateList,
    DbConnection(mut conn): DbConnection,
    form: Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&csrf_tokens) {
        Err(_) => {
            // csrf token is invalid => back to edit view
            Ok(Redirect::to(&candidate_list.update_path()).into_response())
        }
        Ok(_) => {
            candidate_lists::repository::remove_candidate_list(&mut conn, candidate_list.id)
                .await?;
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
        CsrfTokens, DbConnection, ElectoralDistrict, TokenValue,
        candidate_lists::{self, CandidateListId},
    };

    #[sqlx::test]
    async fn delete_candidate_list_and_redirect(pool: PgPool) -> Result<(), sqlx::Error> {
        let mut conn = pool.acquire().await.unwrap();
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let candidate_list = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        };
        candidate_lists::repository::create_candidate_list(&mut conn, &candidate_list).await?;

        let response = delete_candidate_list(
            CandidateListsDeletePath {
                id: candidate_list.id,
            },
            csrf_tokens,
            candidate_list.clone(),
            DbConnection(conn),
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
        let mut conn = pool.acquire().await?;
        let lists = candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(lists.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_candidate_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let mut conn = pool.acquire().await.unwrap();
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = TokenValue("invalid".to_string());
        let candidate_list = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        };
        candidate_lists::repository::create_candidate_list(&mut conn, &candidate_list).await?;

        let response = delete_candidate_list(
            CandidateListsDeletePath {
                id: candidate_list.id,
            },
            csrf_tokens,
            candidate_list.clone(),
            DbConnection(conn),
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
        let mut conn = pool.acquire().await?;
        let lists = candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(lists.len(), 1);

        Ok(())
    }
}
