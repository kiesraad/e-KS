use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context,
    candidate_lists::{CandidateList, pages::CandidateListsDeletePath},
    form::{EmptyForm, Validate},
};

pub async fn delete_candidate_list(
    _: CandidateListsDeletePath,
    context: Context,
    candidate_list: CandidateList,
    State(store): State<AppStore>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => Ok(Redirect::to(&candidate_list.update_path()).into_response()),
        Ok(_) => {
            CandidateList::delete_by_id(&store, candidate_list.id).await?;
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

    use crate::{AppStore, ElectoralDistrict, TokenValue, candidate_lists::CandidateListSummary};

    #[sqlx::test]
    async fn delete_candidate_list_and_redirect(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let candidate_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            ..Default::default()
        };
        candidate_list.create(&store).await?;

        let response = delete_candidate_list(
            CandidateListsDeletePath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(store.clone()),
            Form(EmptyForm { csrf_token }),
        )
        .await?;

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
        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(lists.len(), 0);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_candidate_invalid_form_renders_template(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let context = Context::new_test_without_db();
        let csrf_token = TokenValue("invalid".to_string());
        let candidate_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            ..Default::default()
        };
        candidate_list.create(&store).await?;

        let response = delete_candidate_list(
            CandidateListsDeletePath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(store.clone()),
            Form(EmptyForm { csrf_token }),
        )
        .await?;

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
        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(lists.len(), 1);

        Ok(())
    }
}
