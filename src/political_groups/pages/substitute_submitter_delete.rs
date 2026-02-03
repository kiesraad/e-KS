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

use super::{SubstituteSubmitterDeletePath, SubstituteSubmitterEditPath};

pub async fn delete_substitute_submitter(
    SubstituteSubmitterDeletePath { submitter_id }: SubstituteSubmitterDeletePath,
    political_group: PoliticalGroup,
    context: Context,
    State(pool): State<PgPool>,
    Form(form): Form<EmptyForm>,
) -> Result<Response, AppError> {
    match form.validate_create(&context.csrf_tokens) {
        Err(_) => Ok(
            Redirect::to(&SubstituteSubmitterEditPath { submitter_id }.to_string()).into_response(),
        ),
        Ok(_) => {
            political_groups::remove_substitute_submitter(&pool, political_group.id, submitter_id)
                .await?;

            Ok(Redirect::to(&political_groups::SubstituteSubmitter::list_path()).into_response())
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
        political_groups::{self, PoliticalGroupId, SubstituteSubmitter, SubstituteSubmitterId},
        test_utils::{sample_political_group, sample_substitute_submitter},
    };

    #[sqlx::test]
    async fn delete_substitute_submitter_removes_and_redirects(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_substitute_submitter(
            &pool,
            political_group.id,
            &substitute_submitter,
        )
        .await?;
        political_groups::update_political_group(&pool, &political_group).await?;

        let context = Context::new_test(pool.clone()).await;
        let csrf_token = context.csrf_tokens.issue().value;

        let response = delete_substitute_submitter(
            SubstituteSubmitterDeletePath { submitter_id },
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
        assert_eq!(location, SubstituteSubmitter::list_path());

        let submitters = political_groups::get_substitute_submitters(&pool, group_id).await?;
        assert!(submitters.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn delete_substitute_submitter_invalid_csrf_redirects_to_edit(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let group_id = PoliticalGroupId::new();
        let political_group = sample_political_group(group_id);
        let submitter_id = SubstituteSubmitterId::new();
        let substitute_submitter = sample_substitute_submitter(submitter_id);

        political_groups::create_political_group(&pool, &political_group).await?;
        political_groups::create_substitute_submitter(
            &pool,
            political_group.id,
            &substitute_submitter,
        )
        .await?;
        political_groups::update_political_group(&pool, &political_group).await?;

        let context = Context::new_test(pool.clone()).await;

        let response = delete_substitute_submitter(
            SubstituteSubmitterDeletePath { submitter_id },
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
        assert_eq!(
            location,
            SubstituteSubmitterEditPath { submitter_id }.to_string()
        );

        let submitters = political_groups::get_substitute_submitters(&pool, group_id).await?;
        assert_eq!(submitters.len(), 1);

        Ok(())
    }
}
