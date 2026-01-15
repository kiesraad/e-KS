use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppState, Context, CsrfTokens, DbConnection, ElectionConfig, ElectoralDistrict,
    HtmlTemplate, Locale,
    candidate_lists::{
        self, CandidateList, CandidateListForm, CandidateListSummary,
        pages::{CandidateListsEditPath, candidate_list_not_found},
    },
    filters,
    form::{FormData, Validate},
    persons::{self, Person},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/update.html")]
struct CandidateListUpdateTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    election: ElectionConfig,
    total_persons: i64,
    locale: Locale,
    form: FormData<CandidateListForm>,
    candidate_list: CandidateList,
    electoral_districts: &'static [ElectoralDistrict],
}

pub async fn edit_candidate_list(
    CandidateListsEditPath { id }: CandidateListsEditPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    State(app_state): State<AppState>,
) -> Result<Response, AppError> {
    let candidate_lists =
        candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
    let total_persons = persons::repository::count_persons(&mut conn).await?;
    let election = app_state.config().election;
    let electoral_districts = election.electoral_districts();

    let candidate_list = candidate_lists::repository::get_candidate_list(&mut conn, &id)
        .await?
        .ok_or(candidate_list_not_found(id, context.locale))?;

    Ok(HtmlTemplate(
        CandidateListUpdateTemplate {
            form: FormData::new_with_data(
                CandidateListForm::from(candidate_list.clone()),
                &csrf_tokens,
            ),
            candidate_lists,
            election,
            total_persons,
            locale: context.locale,
            candidate_list,
            electoral_districts,
        },
        context,
    )
    .into_response())
}

pub async fn update_candidate_list(
    CandidateListsEditPath { id }: CandidateListsEditPath,
    context: Context,
    State(app_state): State<AppState>,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    form: Form<CandidateListForm>,
) -> Result<Response, AppError> {
    let candidate_lists =
        candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
    let total_persons = persons::repository::count_persons(&mut conn).await?;
    let election = app_state.config().election;

    let electoral_districts = election.electoral_districts();

    let candidate_list = candidate_lists::repository::get_candidate_list(&mut conn, &id)
        .await?
        .ok_or(candidate_list_not_found(id, context.locale))?;

    match form.validate(Some(&candidate_list), &csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            CandidateListUpdateTemplate {
                candidate_lists,
                election,
                total_persons,
                locale: context.locale,
                form: form_data,
                candidate_list,
                electoral_districts,
            },
            context,
        )
        .into_response()),
        Ok(candidate_list) => {
            let candidate_list =
                candidate_lists::repository::update_candidate_list(&mut conn, &candidate_list)
                    .await?;
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
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
        candidate_lists, test_utils::response_body_string,
    };

    #[sqlx::test]
    async fn update_candidate_list_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let mut conn = pool.acquire().await.unwrap();
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let creation_date = DateTime::from_timestamp(0, 0).unwrap();
        let candidate_list = CandidateList {
            id: Uuid::new_v4(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: creation_date,
            updated_at: creation_date,
        };
        candidate_lists::repository::create_candidate_list(&mut conn, &candidate_list).await?;

        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::DR],
            csrf_token,
        };
        let response = update_candidate_list(
            CandidateListsEditPath {
                id: candidate_list.id,
            },
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
            DbConnection(conn),
            Form(form),
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

        // verify updated candidate list object in database
        let mut conn = pool.acquire().await?;
        let lists = candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        assert_eq!(updated_list.view_path(), location);

        assert_eq!(candidate_list.id, updated_list.id);
        assert_eq!(
            vec![ElectoralDistrict::DR],
            updated_list.electoral_districts
        );
        assert_eq!(creation_date, updated_list.created_at);
        // we don't know the exact update date
        // best we can do is to check it at least got updated (i.e. not equal to creation_date)
        assert_ne!(creation_date, updated_list.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_list_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let mut conn = pool.acquire().await.unwrap();
        let csrf_tokens = CsrfTokens::default();
        let creation_date = DateTime::from_timestamp(0, 0).unwrap();
        let candidate_list = CandidateList {
            id: Uuid::new_v4(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: creation_date,
            updated_at: creation_date,
        };
        candidate_lists::repository::create_candidate_list(&mut conn, &candidate_list).await?;

        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::DR],
            csrf_token: TokenValue("invalid".to_string()),
        };
        let response = update_candidate_list(
            CandidateListsEditPath {
                id: candidate_list.id,
            },
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
            DbConnection(conn),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("Edit candidate list"));

        let mut conn = pool.acquire().await?;
        let lists = candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        // verify candidate list didn't update in database
        assert_eq!(&candidate_list, updated_list);

        Ok(())
    }
}
