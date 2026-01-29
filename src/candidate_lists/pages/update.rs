use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, Context, ElectionConfig, HtmlTemplate,
    candidate_lists::{
        self, CandidateList, CandidateListForm, CandidateListSummary, pages::CandidateListsEditPath,
    },
    filters,
    form::{FormData, Validate},
    persons::{self, Person},
};

#[derive(Template)]
#[template(path = "candidate_lists/update.html")]
struct CandidateListUpdateTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    total_persons: i64,
    form: FormData<CandidateListForm>,
    candidate_list: CandidateList,
}

pub async fn edit_candidate_list(
    _: CandidateListsEditPath,
    context: Context,
    candidate_list: CandidateList,
    State(pool): State<PgPool>,
) -> Result<Response, AppError> {
    let candidate_lists = candidate_lists::list_candidate_list_summary(&pool).await?;
    let total_persons = persons::count_persons(&pool).await?;

    Ok(HtmlTemplate(
        CandidateListUpdateTemplate {
            form: FormData::new_with_data(
                CandidateListForm::from(candidate_list.clone()),
                &context.csrf_tokens,
            ),
            candidate_lists,
            total_persons,
            candidate_list,
        },
        context,
    )
    .into_response())
}

pub async fn update_candidate_list(
    _: CandidateListsEditPath,
    context: Context,
    candidate_list: CandidateList,
    State(pool): State<PgPool>,
    Form(form): Form<CandidateListForm>,
) -> Result<Response, AppError> {
    let candidate_lists = candidate_lists::list_candidate_list_summary(&pool).await?;
    let total_persons = persons::count_persons(&pool).await?;

    match form.validate_update(&candidate_list, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            CandidateListUpdateTemplate {
                candidate_lists,
                total_persons,
                form: form_data,
                candidate_list,
            },
            context,
        )
        .into_response()),
        Ok(candidate_list) => {
            let candidate_list =
                candidate_lists::update_candidate_list(&pool, &candidate_list).await?;
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{StatusCode, header};
    use axum_extra::extract::Form;
    use chrono::{DateTime, Duration, Utc};
    use sqlx::PgPool;

    use crate::{
        Context, ElectoralDistrict, TokenValue,
        candidate_lists::{self, CandidateListId},
        test_utils::{response_body_string, sample_candidate_list},
    };

    #[sqlx::test]
    async fn edit_candidate_list_renders_existing_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let candidate_list = sample_candidate_list(CandidateListId::new());

        candidate_lists::create_candidate_list(&pool, &candidate_list).await?;

        let response = edit_candidate_list(
            CandidateListsEditPath {
                list_id: candidate_list.id,
            },
            Context::new_test(pool.clone()).await,
            candidate_list.clone(),
            State(pool.clone()),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Edit candidate list"));
        assert!(body.contains(&candidate_list.update_path()));
        assert!(body.contains("electoral_district_UT"));
        assert!(body.contains("checked"));

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_list_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let creation_date = DateTime::from_timestamp(0, 0).unwrap();
        let candidate_list = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: creation_date,
            updated_at: creation_date,
        };
        let candidate_list = candidate_lists::create_candidate_list(&pool, &candidate_list).await?;

        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::DR],
            csrf_token,
        };
        let response = update_candidate_list(
            CandidateListsEditPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(pool.clone()),
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
        let lists = candidate_lists::list_candidate_list_summary(&pool).await?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        assert_eq!(updated_list.view_path(), location);

        assert_eq!(candidate_list.id, updated_list.id);
        assert_eq!(
            vec![ElectoralDistrict::DR],
            updated_list.electoral_districts
        );
        assert!((candidate_list.created_at - Utc::now()).abs() < Duration::seconds(10));
        // we don't know the exact update date
        // best we can do is to check it at least got updated (i.e. not equal to creation_date)
        assert_ne!(candidate_list.created_at, updated_list.updated_at);

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_list_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let creation_date = DateTime::from_timestamp(0, 0).unwrap();
        let candidate_list = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: creation_date,
            updated_at: creation_date,
        };
        candidate_lists::create_candidate_list(&pool, &candidate_list).await?;

        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::DR],
            csrf_token: TokenValue("invalid".to_string()),
        };
        let response = update_candidate_list(
            CandidateListsEditPath {
                list_id: candidate_list.id,
            },
            Context::new_test(pool.clone()).await,
            candidate_list.clone(),
            State(pool.clone()),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("Edit candidate list"));

        let lists = candidate_lists::list_candidate_list_summary(&pool).await?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        // verify candidate list didn't update in database
        assert_eq!(
            candidate_list.electoral_districts,
            updated_list.electoral_districts
        );

        Ok(())
    }
}
