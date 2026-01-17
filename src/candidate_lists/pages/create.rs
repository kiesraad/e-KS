use askama::Template;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::Form;

use crate::{
    AppError, Context, CsrfTokens, DbConnection, ElectionConfig, HtmlTemplate,
    candidate_lists::{
        self, CandidateList, CandidateListForm, CandidateListSummary, pages::CandidateListNewPath,
    },
    filters,
    form::{FormData, Validate},
    persons,
    persons::Person,
    t,
};

#[derive(Template)]
#[template(path = "candidate_lists/create.html")]
struct CandidateListCreateTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    total_persons: i64,
    form: FormData<CandidateListForm>,
}

pub async fn new_candidate_list_form(
    _: CandidateListNewPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
) -> Result<impl IntoResponse, AppError> {
    let candidate_lists =
        candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
    let total_persons = persons::repository::count_persons(&mut conn).await?;
    let used_districts = candidate_lists::repository::get_used_districts(&mut conn).await?;
    let available_districts = context.election.available_districts(used_districts);

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
            total_persons,
            form,
        },
        context,
    )
    .into_response())
}

pub async fn create_candidate_list(
    _: CandidateListNewPath,
    context: Context,
    csrf_tokens: CsrfTokens,
    DbConnection(mut conn): DbConnection,
    Form(form): Form<CandidateListForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&csrf_tokens) {
        Err(form_data) => {
            let candidate_lists =
                candidate_lists::repository::list_candidate_list_with_count(&mut conn).await?;
            let total_persons = persons::repository::count_persons(&mut conn).await?;

            Ok(HtmlTemplate(
                CandidateListCreateTemplate {
                    candidate_lists,
                    total_persons,
                    form: form_data,
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
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        Context, CsrfTokens, DbConnection, ElectoralDistrict, Locale, TokenValue, candidate_lists,
        test_utils::response_body_string,
    };

    #[sqlx::test]
    async fn new_candidate_list_form_renders_csrf_field(pool: PgPool) -> Result<(), sqlx::Error> {
        let response = new_candidate_list_form(
            CandidateListNewPath {},
            Context::new(Locale::En),
            CsrfTokens::default(),
            DbConnection(pool.acquire().await?),
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
        let csrf_tokens = CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            csrf_token,
        };

        let response = create_candidate_list(
            CandidateListNewPath {},
            Context::new(Locale::En),
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
        let csrf_tokens = CsrfTokens::default();
        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            csrf_token: TokenValue("invalid".to_string()),
        };

        let response = create_candidate_list(
            CandidateListNewPath {},
            Context::new(Locale::En),
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
        let election = ElectionConfig::EK2027;
        let all_districts = election.electoral_districts().to_vec();

        let none_used = vec![];
        let all_used = all_districts.clone();
        let some_used = vec![
            ElectoralDistrict::DR,
            ElectoralDistrict::FL,
            ElectoralDistrict::FR,
            ElectoralDistrict::GE,
            ElectoralDistrict::GR,
            ElectoralDistrict::LI,
            ElectoralDistrict::NB,
            ElectoralDistrict::NH,
        ];

        // use sets so we don't need to worry about ordering of the vector
        let none_used_result: BTreeSet<ElectoralDistrict> = election
            .available_districts(none_used)
            .into_iter()
            .collect();
        let all_used_result: BTreeSet<ElectoralDistrict> =
            election.available_districts(all_used).into_iter().collect();
        let some_used_result: BTreeSet<ElectoralDistrict> = election
            .available_districts(some_used)
            .into_iter()
            .collect();

        // validation
        let all_district_set: BTreeSet<ElectoralDistrict> = all_districts.into_iter().collect();
        assert_eq!(all_district_set, none_used_result);
        assert_eq!(BTreeSet::new(), all_used_result);
        assert_eq!(
            BTreeSet::from([
                ElectoralDistrict::OV,
                ElectoralDistrict::UT,
                ElectoralDistrict::ZE,
                ElectoralDistrict::ZH,
                ElectoralDistrict::BO,
                ElectoralDistrict::SE,
                ElectoralDistrict::SA,
                ElectoralDistrict::KN,
            ]),
            some_used_result
        );
    }
}
