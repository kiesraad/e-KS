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
        self, CandidateList, CandidateListForm, CandidateListSummary, pages::CandidateListNewPath,
    },
    filters,
    form::{FormData, Validate},
    persons::{self, Person},
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/create.html")]
struct CandidateListCreateTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    election: ElectionConfig,
    total_persons: i64,
    form: FormData<CandidateListForm>,
    locale: Locale,
    electoral_districts: &'static [ElectoralDistrict],
}

pub async fn new_candidate_list_form(
    _: CandidateListNewPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let candidate_lists =
        candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
    let total_persons = persons::repository::count_persons(&mut conn).await?;
    let election = app_state.config().election;

    let electoral_districts = app_state.config().get_districts();

    let used_districts = candidate_lists::repository::get_used_districts(&mut conn).await?;
    let available_districts: Vec<ElectoralDistrict> =
        determine_available_districts(electoral_districts, used_districts);

    let form = FormData::new_with_data(
        CandidateListForm {
            electoral_districts: available_districts,
            csrf_token: csrf_tokens.issue().value,
        },
        &csrf_tokens,
    );

    Ok(HtmlTemplate(
        CandidateListCreateTemplate {
            candidate_lists,
            election,
            total_persons,
            form,
            locale: context.locale,
            electoral_districts,
        },
        context,
    )
    .into_response())
}

fn determine_available_districts(
    electoral_districts: &[ElectoralDistrict],
    used_districts: Vec<ElectoralDistrict>,
) -> Vec<ElectoralDistrict> {
    electoral_districts
        .iter()
        .filter(|d| !used_districts.contains(d))
        .cloned()
        .collect()
}

pub async fn create_candidate_list(
    _: CandidateListNewPath,
    context: Context,
    State(app_state): State<AppState>,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<CandidateListForm>,
) -> Result<Response, AppError> {
    let electoral_districts = app_state.config().election.electoral_districts();

    match form.validate(None, &csrf_tokens) {
        Err(form_data) => {
            let candidate_lists =
                candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
            let total_persons = persons::repository::count_persons(&mut conn).await?;
            let election = app_state.config().election;

            Ok(HtmlTemplate(
                CandidateListCreateTemplate {
                    candidate_lists,
                    election,
                    total_persons,
                    form: form_data,
                    electoral_districts,
                    locale: context.locale,
                },
                context,
            )
            .into_response())
        }
        Ok(candidate_list) => {
            let candidate_list =
                candidate_lists::repository::create_candidate_list(&mut conn, &candidate_list)
                    .await?;
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use super::*;
    use axum::{
        extract::State,
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        AppState, Context, CsrfTokens, DbConnection, Locale, TokenValue, candidate_lists,
        test_utils::response_body_string,
    };

    #[sqlx::test]
    async fn new_candidate_list_form_renders_csrf_field(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());

        let response = new_candidate_list_form(
            CandidateListNewPath {},
            Context::new(Locale::En),
            CsrfTokens::default(),
            DbConnection(pool.acquire().await?),
            State(app_state),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("name=\"csrf_token\""));
        assert!(body.contains("action=\"/candidate-lists/new\""));

        Ok(())
    }

    #[sqlx::test]
    async fn create_candidate_list_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            csrf_token,
        };

        let response = create_candidate_list(
            CandidateListNewPath {},
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
            DbConnection(pool.acquire().await?),
            Form(form),
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

        let mut conn = pool.acquire().await?;
        let lists = candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(lists.len(), 1);
        assert_eq!(location, lists[0].list.view_path());

        Ok(())
    }

    #[sqlx::test]
    async fn create_candidate_list_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let app_state = AppState::new_for_tests(pool.clone());
        let csrf_tokens = CsrfTokens::default();
        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            csrf_token: TokenValue("invalid".to_string()),
        };

        let response = create_candidate_list(
            CandidateListNewPath {},
            Context::new(Locale::En),
            State(app_state),
            csrf_tokens,
            DbConnection(pool.acquire().await?),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("Create candidate list"));

        Ok(())
    }

    #[test]
    fn test_determine_available_districts() {
        // setup
        let all_districts = vec![
            ElectoralDistrict::DR,
            ElectoralDistrict::FR,
            ElectoralDistrict::UT,
            ElectoralDistrict::OV,
        ];

        let none_used = vec![];
        let all_used = all_districts.clone();
        let some_used = vec![ElectoralDistrict::DR, ElectoralDistrict::FR];

        // test
        // use sets so we don't need to worry about ordering of the vector
        let none_used_result: BTreeSet<ElectoralDistrict> =
            determine_available_districts(&all_districts, none_used)
                .into_iter()
                .collect();
        let all_used_result: BTreeSet<ElectoralDistrict> =
            determine_available_districts(&all_districts, all_used)
                .into_iter()
                .collect();
        let some_used_result: BTreeSet<ElectoralDistrict> =
            determine_available_districts(&all_districts, some_used)
                .into_iter()
                .collect();

        // validation
        let all_district_set: BTreeSet<ElectoralDistrict> = all_districts.into_iter().collect();
        assert_eq!(all_district_set, none_used_result);
        assert_eq!(BTreeSet::new(), all_used_result);
        assert_eq!(
            BTreeSet::from([ElectoralDistrict::UT, ElectoralDistrict::OV]),
            some_used_result
        );
    }
}
