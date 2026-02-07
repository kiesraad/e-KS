use askama::Template;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;

use crate::{
    AppError, AppStore, Context, ElectionConfig, HtmlTemplate, InitialQuery,
    candidate_lists::{CandidateList, CandidateListForm, pages::CandidateListUpdatePath},
    filters,
    form::FormData,
};

#[derive(Template)]
#[template(path = "candidate_lists/update.html")]
struct CandidateListUpdateTemplate {
    should_warn: bool,
    form: FormData<CandidateListForm>,
    candidate_list: CandidateList,
}

pub async fn update_candidate_list(
    _: CandidateListUpdatePath,
    context: Context,
    candidate_list: CandidateList,
    Query(query): Query<InitialQuery>,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(
        CandidateListUpdateTemplate {
            form: FormData::new_with_data(
                CandidateListForm::from(candidate_list.clone()),
                &context.csrf_tokens,
            ),
            should_warn: query.should_warn(),
            candidate_list,
        },
        context,
    )
    .into_response())
}

pub async fn update_candidate_list_submit(
    _: CandidateListUpdatePath,
    context: Context,
    candidate_list: CandidateList,
    State(store): State<AppStore>,
    Query(query): Query<InitialQuery>,
    Form(form): Form<CandidateListForm>,
) -> Result<Response, AppError> {
    match form.validate_update(&candidate_list, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            CandidateListUpdateTemplate {
                should_warn: query.should_warn(),
                form: form_data,
                candidate_list,
            },
            context,
        )
        .into_response()),
        Ok(candidate_list) => {
            candidate_list.update(&store).await?;
            Ok(Redirect::to(&candidate_list.view_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore, Context, ElectoralDistrict, InitialQuery, TokenValue, UtcDateTime,
        candidate_lists::{CandidateListId, CandidateListSummary},
        test_utils::{response_body_string, sample_candidate_list},
    };
    use axum::{
        extract::Query,
        http::{StatusCode, header},
    };
    use axum_extra::extract::Form;
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn update_candidate_list_renders_existing_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let candidate_list = sample_candidate_list(CandidateListId::new());

        candidate_list.create(&store).await?;

        let response = update_candidate_list(
            CandidateListUpdatePath {
                list_id: candidate_list.id,
            },
            Context::new_test_without_db(),
            candidate_list.clone(),
            Query(InitialQuery::default()),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Edit candidate list"));
        assert!(body.contains(&candidate_list.update_path()));
        assert!(body.contains("electoral_district_UT"));
        assert!(body.contains("checked"));

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_list_persists_and_redirects() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let context = Context::new_test_without_db();
        let csrf_token = context.csrf_tokens.issue().value;
        let candidate_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            updated_at: UtcDateTime::now(),
            ..Default::default()
        };
        candidate_list.create(&store).await?;

        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::DR],
            csrf_token,
        };
        let response = update_candidate_list_submit(
            CandidateListUpdatePath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(store.clone()),
            Query(InitialQuery::default()),
            Form(form),
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

        // verify updated candidate list object in database
        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(lists.len(), 1);

        let updated_list = &lists[0].list;

        assert_eq!(updated_list.view_path(), location);

        assert_eq!(candidate_list.id, updated_list.id);
        assert_eq!(
            vec![ElectoralDistrict::DR],
            updated_list.electoral_districts
        );

        assert!(
            Utc::now() - chrono::DateTime::<Utc>::from(candidate_list.updated_at)
                < Duration::seconds(10)
        );

        // we don't know the exact update date
        // best we can do is to check it at least got updated (i.e. not equal to creation_date)
        assert_ne!(candidate_list.created_at, updated_list.updated_at);

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_list_invalid_form_renders_template() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let creation_date = UtcDateTime::now();
        let candidate_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: creation_date,
            updated_at: creation_date,
            ..Default::default()
        };
        candidate_list.create(&store).await?;

        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::DR],
            csrf_token: TokenValue("invalid".to_string()),
        };
        let response = update_candidate_list_submit(
            CandidateListUpdatePath {
                list_id: candidate_list.id,
            },
            Context::new_test_without_db(),
            candidate_list.clone(),
            State(store.clone()),
            Query(InitialQuery::default()),
            Form(form),
        )
        .await?;

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("Edit candidate list"));

        let lists = CandidateListSummary::list(&store)?;
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
