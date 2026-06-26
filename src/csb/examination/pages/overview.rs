use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, Context, CsbContext, HtmlTemplate,
    csb::{
        examination::{
            extractors::{CsbPoliticalGroup, CsbPoliticalGroups},
            pages::CsbExaminationOverviewPath,
        },
        import::CsbImportPath,
    },
    filters,
};

#[derive(Template)]
#[template(path = "examination/pages/overview.html")]
struct CsbExaminationOverviewTemplate {
    political_groups: Vec<CsbPoliticalGroup>,
}

/// Render the placeholder overview page.
pub async fn overview(
    _: CsbExaminationOverviewPath,
    context: CsbContext,
    CsbPoliticalGroups(political_groups): CsbPoliticalGroups,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(CsbExaminationOverviewTemplate { political_groups }, context).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::{
        StreamId,
        test_utils::{response_body_string, sample_political_group},
    };

    #[tokio::test]
    async fn overview_renders_imported_political_group_names() {
        let groups = CsbPoliticalGroups(vec![CsbPoliticalGroup {
            political_group: sample_political_group(),
            stream_id: StreamId::new(),
        }]);

        let response = overview(
            CsbExaminationOverviewPath {},
            CsbContext::new_test(),
            groups,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        // The seeded group's display name is rendered in the "added" table.
        let body = response_body_string(response).await;
        assert!(body.contains("Kiesraad Demo"));
    }

    #[tokio::test]
    async fn overview_renders_without_political_groups() {
        let response = overview(
            CsbExaminationOverviewPath {},
            CsbContext::new_test(),
            CsbPoliticalGroups(vec![]),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
