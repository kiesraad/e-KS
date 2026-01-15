use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection,
    candidate_lists::{
        self,
        pages::{CandidateListsDeletePath, candidate_list_not_found},
        structs::CandidateList,
    },
    form::{EmptyForm, Validate},
};

pub(crate) async fn delete_candidate_list(
    CandidateListsDeletePath { id }: CandidateListsDeletePath,
    context: Context,
    _: State<AppState>,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    form: Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate(None, &csrf_tokens) {
        Err(_) => {
            // csrf token is invalid => back to edit view
            let candidate_list = candidate_lists::repository::get_candidate_list(&mut conn, &id)
                .await?
                .ok_or(candidate_list_not_found(id, context.locale))?;
            Ok(Redirect::to(&candidate_list.update_path()).into_response())
        }
        Ok(_) => {
            candidate_lists::repository::remove_candidate_list(&mut conn, id).await?;
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
    use chrono::DateTime;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        AppState, Context, CsrfTokens, DbConnection, ElectoralDistrict, Locale, TokenValue,
        candidate_lists,
    };

    #[sqlx::test]
    async fn delete_candidate_list_and_redirect(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let mut conn = pool.acquire().await.unwrap();
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let candidate_list = CandidateList {
            id: Uuid::new_v4(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        };
        candidate_lists::repository::create_candidate_list(&mut conn, &candidate_list).await?;

        let response = delete_candidate_list(
            CandidateListsDeletePath {
                id: candidate_list.id,
            },
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
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
        let app_state = AppState::new_for_tests(pool.clone());
        let mut conn = pool.acquire().await.unwrap();
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = TokenValue("invalid".to_string());
        let candidate_list = CandidateList {
            id: Uuid::new_v4(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        };
        candidate_lists::repository::create_candidate_list(&mut conn, &candidate_list).await?;

        let response = delete_candidate_list(
            CandidateListsDeletePath {
                id: candidate_list.id,
            },
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
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
