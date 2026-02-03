use super::SubstituteSubmittersPath;
use crate::{
    AppError, Context, HtmlTemplate, filters,
    political_groups::{self, AuthorisedAgent, ListSubmitter, PoliticalGroup, SubstituteSubmitter},
};
use askama::Template;
use axum::{extract::State, response::IntoResponse};
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "political_groups/substitute_submitters.html")]
struct SubstituteSubmittersTemplate {
    substitute_submitters: Vec<SubstituteSubmitter>,
}

pub async fn list_substitute_submitters(
    _: SubstituteSubmittersPath,
    context: Context,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AppError> {
    let substitute_submitters =
        political_groups::get_substitute_submitters(&pool, political_group.id).await?;

    Ok(HtmlTemplate(
        SubstituteSubmittersTemplate {
            substitute_submitters,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};
    use sqlx::PgPool;

    use crate::{
        Context,
        political_groups::{self, PoliticalGroupId, SubstituteSubmitterId},
        test_utils::{response_body_string, sample_political_group, sample_substitute_submitter},
    };

    #[sqlx::test]
    async fn list_substitute_submitters_shows_created_submitter(
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

        let response = list_substitute_submitters(
            SubstituteSubmittersPath {},
            Context::new_test(pool.clone()).await,
            political_group,
            State(pool.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&substitute_submitter.last_name));

        Ok(())
    }

    #[sqlx::test]
    async fn list_substitute_submitters_shows_edit_link(pool: PgPool) -> Result<(), sqlx::Error> {
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

        let response = list_substitute_submitters(
            SubstituteSubmittersPath {},
            Context::new_test(pool.clone()).await,
            political_group,
            State(pool.clone()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&substitute_submitter.edit_path()));

        Ok(())
    }
}
