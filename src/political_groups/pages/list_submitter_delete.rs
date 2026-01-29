use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use sqlx::PgPool;

use crate::{
    AppError, Context,
    form::{EmptyForm, Validate},
    political_groups::{self, PoliticalGroup},
};

use super::{ListSubmitterDeletePath, ListSubmitterEditPath};

pub async fn delete_list_submitter(
    ListSubmitterDeletePath { submitter_id }: ListSubmitterDeletePath,
    political_group: PoliticalGroup,
    context: Context,
    State(pool): State<PgPool>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => {
            Ok(Redirect::to(&ListSubmitterEditPath { submitter_id }.to_string()).into_response())
        }
        Ok(_) => {
            political_groups::remove_list_submitter(&pool, political_group.id, submitter_id)
                .await?;

            Ok(Redirect::to(&political_groups::ListSubmitter::list_path()).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_extra::extract::Form;
    use sqlx::PgPool;

    use crate::{
        Context, TokenValue,
        political_groups::{self, ListSubmitter, ListSubmitterId, PoliticalGroupId},
        test_utils::{sample_list_submitter, sample_political_group},
    };

    #[sqlx::test]
    async fn delete_list_submitter_removes_and_redirects(pool: PgPool) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let mut political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_list_submitter(&pool, political_group.id, &list_submitter).await?;
        political_group.list_submitter_id = Some(submitter_id);
        political_groups::update_political_group(&pool, &political_group).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_list_submitter(
            ListSubmitterDeletePath { submitter_id },
            political_group,
            context,
            State(pool.clone()),
            Form(EmptyForm::new(csrf_token)),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(location, ListSubmitter::list_path());

        let submitters = political_groups::get_list_submitters(&pool, group_id).await?;
        assert!(submitters.is_empty());

        let political_group = political_groups::get_single_political_group(&pool)
            .await?
            .expect("political group");
        assert!(political_group.list_submitter_id.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn delete_list_submitter_invalid_csrf_redirects_to_edit(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let mut political_group = sample_political_group(group_id);
        let submitter_id = ListSubmitterId::new();
        let list_submitter = sample_list_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_list_submitter(&pool, political_group.id, &list_submitter).await?;
        political_group.list_submitter_id = Some(submitter_id);
        political_groups::update_political_group(&pool, &political_group).await?;

        let context = Context::new_test(pool.clone()).await;

        let response = delete_list_submitter(
            ListSubmitterDeletePath { submitter_id },
            political_group,
            context,
            State(pool.clone()),
            Form(EmptyForm::new(TokenValue("invalid".to_string()))),
        )
        .await
        .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .expect("location header")
            .to_str()
            .expect("location header value");
        assert_eq!(location, ListSubmitterEditPath { submitter_id }.to_string());

        let submitters = political_groups::get_list_submitters(&pool, group_id).await?;
        assert_eq!(submitters.len(), 1);

        let political_group = political_groups::get_single_political_group(&pool)
            .await?
            .expect("political group");
        assert_eq!(political_group.list_submitter_id, Some(submitter_id));

        Ok(())
    }
}
