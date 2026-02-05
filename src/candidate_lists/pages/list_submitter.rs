use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, Context, HtmlTemplate,
    candidate_lists::{
        self, CandidateList, CandidateListSummary, ListSubmitterForm, pages::EditListSubmitterPath,
    },
    filters,
    form::{FormData, Validate},
    persons::{self, Person},
    political_groups::{self, ListSubmitter},
};

#[derive(Template)]
#[template(path = "candidate_lists/list_submitter.html")]
struct ListSubmitterUpdateTemplate {
    candidate_lists: Vec<CandidateListSummary>,
    total_persons: i64,
    form: FormData<ListSubmitterForm>,
    candidate_list: CandidateList,
    list_submitters: Vec<ListSubmitter>,
}

pub async fn edit_list_submitter_form(
    _: EditListSubmitterPath,
    context: Context,
    candidate_list: CandidateList,
    State(pool): State<PgPool>,
) -> Result<Response, AppError> {
    let candidate_lists = candidate_lists::list_candidate_list_summary(&pool).await?;
    let total_persons = persons::count_persons(&pool).await?;
    let list_submitters =
        political_groups::get_list_submitters(&pool, context.political_group.id).await?;

    let form = FormData::new_with_data(
        ListSubmitterForm::from(candidate_list.clone()),
        &context.csrf_tokens,
    );

    Ok(HtmlTemplate(
        ListSubmitterUpdateTemplate {
            candidate_lists,
            total_persons,
            form,
            candidate_list,
            list_submitters,
        },
        context,
    )
    .into_response())
}

pub async fn update_list_submitter(
    _: EditListSubmitterPath,
    context: Context,
    candidate_list: CandidateList,
    State(pool): State<PgPool>,
    Form(form): Form<ListSubmitterForm>,
) -> Result<Response, AppError> {
    let candidate_lists = candidate_lists::list_candidate_list_summary(&pool).await?;
    let total_persons = persons::count_persons(&pool).await?;
    let list_submitters =
        political_groups::get_list_submitters(&pool, context.political_group.id).await?;
    match form.validate_update(&candidate_list, &context.csrf_tokens) {
        Err(form_data) => Ok(HtmlTemplate(
            ListSubmitterUpdateTemplate {
                candidate_lists,
                total_persons,
                form: form_data,
                candidate_list,
                list_submitters,
            },
            context,
        )
        .into_response()),
        Ok(candidate_list) => {
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
    use chrono::DateTime;
    use sqlx::PgPool;

    use crate::{
        Context, ElectoralDistrict, TokenValue,
        candidate_lists::{self, CandidateListId},
        political_groups::{ListSubmitterId, PoliticalGroupId},
        test_utils::{
            response_body_string, sample_candidate_list, sample_list_submitter,
            sample_political_group,
        },
    };

    #[sqlx::test]
    async fn edit_list_submitter_renders_list_submitter_form(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let candidate_list = sample_candidate_list(CandidateListId::new());
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let political_group = sample_political_group(PoliticalGroupId::new());

        candidate_lists::create_candidate_list(&pool, &candidate_list).await?;
        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_list_submitter(&pool, political_group.id, &list_submitter).await?;

        let response = edit_list_submitter_form(
            EditListSubmitterPath {
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
        assert!(body.contains("Submitter of the list"));
        assert!(body.contains("csrf_token"));
        assert!(body.contains(&candidate_list.edit_list_submitter_path()));
        assert!(body.contains(&list_submitter.last_name));
        assert!(body.contains(&list_submitter.initials));

        Ok(())
    }

    #[sqlx::test]
    async fn update_list_submitter_persists_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;
        let creation_date = DateTime::from_timestamp(0, 0).unwrap();
        let candidate_list = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::UT],
            list_submitter_id: None,
            created_at: creation_date,
            updated_at: creation_date,
        };
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let political_group = sample_political_group(PoliticalGroupId::new());

        candidate_lists::create_candidate_list(&pool, &candidate_list).await?;
        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_list_submitter(&pool, political_group.id, &list_submitter).await?;

        let form = ListSubmitterForm {
            list_submitter_id: list_submitter.id.to_string(),
            csrf_token,
        };
        let response = update_list_submitter(
            EditListSubmitterPath {
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

        assert_eq!(list_submitter.id, updated_list.list_submitter_id.unwrap());

        Ok(())
    }

    #[sqlx::test]
    async fn update_list_submitter_invalid_form_renders_template(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let context = Context::new_test(pool.clone()).await;
        let candidate_list = sample_candidate_list(CandidateListId::new());
        let list_submitter = sample_list_submitter(ListSubmitterId::new());
        let political_group = sample_political_group(PoliticalGroupId::new());

        candidate_lists::create_candidate_list(&pool, &candidate_list).await?;
        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_list_submitter(&pool, political_group.id, &list_submitter).await?;

        let form = ListSubmitterForm {
            list_submitter_id: list_submitter.id.to_string(),
            csrf_token: TokenValue("invalid".to_string()),
        };
        let response = update_list_submitter(
            EditListSubmitterPath {
                list_id: candidate_list.id,
            },
            context,
            candidate_list.clone(),
            State(pool.clone()),
            Form(form),
        )
        .await
        .unwrap();

        assert_eq!(StatusCode::OK, response.status());
        let body = response_body_string(response).await;
        assert!(body.contains("Submitter of the list"));

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
