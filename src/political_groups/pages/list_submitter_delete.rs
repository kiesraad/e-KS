use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use sqlx::PgPool;

use crate::{
    AppError,
    political_groups::{self, PoliticalGroup},
};

use super::ListSubmitterDeletePath;

pub async fn delete_list_submitter(
    ListSubmitterDeletePath { submitter_id }: ListSubmitterDeletePath,
    political_group: PoliticalGroup,
    State(pool): State<PgPool>,
) -> Result<Response, AppError> {
    political_groups::remove_list_submitter(&pool, &political_group.id, submitter_id).await?;

    Ok(Redirect::to(&political_groups::ListSubmitter::list_path()).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    use crate::{
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
        political_groups::create_list_submitter(&pool, &political_group.id, &list_submitter)
            .await?;
        political_group.list_submitter_id = Some(submitter_id);
        political_groups::update_political_group(&pool, &political_group).await?;

        let response = delete_list_submitter(
            ListSubmitterDeletePath { submitter_id },
            political_group,
            State(pool.clone()),
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

        let submitters = political_groups::get_list_submitters(&pool, &group_id).await?;
        assert!(submitters.is_empty());

        let political_group = political_groups::get_single_political_group(&pool)
            .await?
            .expect("political group");
        assert!(political_group.list_submitter_id.is_none());

        Ok(())
    }
}
