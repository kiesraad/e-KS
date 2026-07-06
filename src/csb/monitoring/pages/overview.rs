use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{
    AppError, Context, CsbContext, HtmlTemplate,
    csb::monitoring::{extractors::StreamMonitor, pages::CsbMonitoringOverviewPath},
    filters,
};

#[derive(Template)]
#[template(path = "csb/monitoring/pages/overview.html")]
struct CsbMonitoringOverviewTemplate {
    monitor: StreamMonitor,
}

/// Render the monitoring overview of political-group streams.
pub async fn overview(
    _: CsbMonitoringOverviewPath,
    context: CsbContext,
    monitor: StreamMonitor,
) -> Result<Response, AppError> {
    Ok(HtmlTemplate(CsbMonitoringOverviewTemplate { monitor }, context).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::{
        ElectionConfig, StreamId, csb::monitoring::extractors::StreamMonitorRow, store::StreamMeta,
        test_utils::response_body_string,
    };

    #[tokio::test]
    async fn overview_renders_without_streams() {
        let monitor = StreamMonitor {
            rows: vec![],
            database_enabled: false,
        };

        let response = overview(
            CsbMonitoringOverviewPath {},
            CsbContext::new_test(),
            monitor,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn overview_renders_stream_rows() {
        let monitor = StreamMonitor {
            rows: vec![StreamMonitorRow {
                meta: StreamMeta {
                    stream_id: StreamId::new(),
                    election: ElectionConfig::EK27,
                    event_count: 3,
                    created_at: None,
                    last_event_at: None,
                },
                political_group_name: Some("Kiesraad Demo".to_string()),
                cache_until_event: None,
            }],
            database_enabled: false,
        };

        let response = overview(
            CsbMonitoringOverviewPath {},
            CsbContext::new_test(),
            monitor,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Kiesraad Demo"));
    }
}
