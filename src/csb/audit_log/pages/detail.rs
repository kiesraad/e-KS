use askama::Template;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};

use crate::{
    AppError, AppRequestState, Context, CsbContext, CsbMainStore, Event, HtmlTemplate, Locale,
    Overlay, QueryParamState, csb::audit_log::pages::CsbAuditLogDetailPath, filters,
    projection::CSB_MAIN_STREAM_ID, store::StoreEvent, structs::audit_log::FieldChange, trans,
};

struct CsbEventDetail {
    event_id: usize,
    stream_label: String,
    description: String,
    details: String,
    created_at: DateTime<Utc>,
    changes: Vec<FieldChange>,
}

impl CsbEventDetail {
    /// Look up `event_id` in a stream's events and build its detail view.
    fn find<E: Event>(
        events: &[StoreEvent<E>],
        event_id: usize,
        stream_label: String,
        locale: Locale,
    ) -> Result<Self, AppError> {
        let event = events
            .iter()
            .find(|e| e.event_id == event_id)
            .ok_or(AppError::GenericNotFound)?;
        Ok(Self {
            event_id: event.event_id,
            stream_label,
            description: event.payload.description(locale),
            details: event.payload.details(),
            changes: event.payload.changes(locale),
            created_at: event.created_at,
        })
    }
}

#[derive(Template)]
#[template(path = "csb/audit_log/pages/detail.html")]
struct CsbAuditLogDetailTemplate {
    detail: CsbEventDetail,
    overlay: Overlay,
}

pub async fn csb_audit_log_detail<S: AppRequestState>(
    CsbAuditLogDetailPath {
        stream_id,
        event_id,
    }: CsbAuditLogDetailPath,
    context: CsbContext,
    main_store: CsbMainStore,
    State(state): State<S>,
    Query(query): Query<QueryParamState>,
) -> Result<impl IntoResponse, AppError> {
    let locale = context.session.locale;
    let detail = if stream_id == CSB_MAIN_STREAM_ID {
        let data = main_store.data.read();
        CsbEventDetail::find(
            &data.events,
            event_id,
            trans!("audit_log.filter.csb_main_stream", locale),
            locale,
        )?
    } else {
        let import_stores = state.csb_store_registry().stores_by_scope().await?;
        let store = import_stores
            .iter()
            .find(|s| s.stream_id == stream_id)
            .ok_or(AppError::GenericNotFound)?;
        let data = store.data.read();
        CsbEventDetail::find(
            &data.events,
            event_id,
            store.get_display_name(crate::projection::WithCorrections::All),
            locale,
        )?
    };

    Ok(HtmlTemplate(
        CsbAuditLogDetailTemplate {
            detail,
            overlay: Overlay::new(&query),
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::{Query, State},
        http::StatusCode,
        response::IntoResponse,
    };

    use crate::{
        AppError, AppState, CsbContext, CsbEvent, CsbMainEvent, CsbMainStore, ElectionConfig,
        QueryParamState, StreamId, csb::audit_log::pages::CsbAuditLogDetailPath,
        test_utils::response_body_string,
    };

    #[tokio::test]
    async fn renders_detail_for_main_stream_event() -> Result<(), AppError> {
        let main_store = CsbMainStore::new_for_test();
        main_store
            .update(CsbMainEvent::DeveloperLogin {
                stream_id: CSB_MAIN_STREAM_ID,
            })
            .await?;

        let state = AppState::new_for_tests().await;

        let response = csb_audit_log_detail(
            CsbAuditLogDetailPath {
                stream_id: CSB_MAIN_STREAM_ID,
                event_id: 1,
            },
            CsbContext::new_test(),
            main_store,
            State(state),
            Query(QueryParamState::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Developer login"));

        Ok(())
    }

    #[tokio::test]
    async fn close_link_returns_to_redirect_target() -> Result<(), AppError> {
        let main_store = CsbMainStore::new_for_test();
        main_store
            .update(CsbMainEvent::DeveloperLogin {
                stream_id: CSB_MAIN_STREAM_ID,
            })
            .await?;

        let state = AppState::new_for_tests().await;
        let return_url = "/csb/audit-log?per_page=20&event_type=developer_login";

        let response = csb_audit_log_detail(
            CsbAuditLogDetailPath {
                stream_id: CSB_MAIN_STREAM_ID,
                event_id: 1,
            },
            CsbContext::new_test(),
            main_store,
            State(state),
            Query(QueryParamState::redirect_to(return_url.to_string())),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        // The overlay close link points back at the filtered list, not the bare
        // audit-log path. `&` is HTML-escaped in the rendered attribute.
        assert!(
            body.contains("href=\"/csb/audit-log?per_page=20&#38;event_type=developer_login\"")
        );

        Ok(())
    }

    #[tokio::test]
    async fn renders_detail_for_import_stream_event() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let stream_id = StreamId::new();
        let csb_store = state
            .csb_store_for_stream(stream_id, ElectionConfig::EK27)
            .await?;
        csb_store.update(CsbEvent::SetFinished(true)).await?;

        let response = csb_audit_log_detail(
            CsbAuditLogDetailPath {
                stream_id,
                event_id: 1,
            },
            CsbContext::new_test(),
            CsbMainStore::new_for_test(),
            State(state),
            Query(QueryParamState::default()),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Set finished state"));

        Ok(())
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_event() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;

        let result = csb_audit_log_detail(
            CsbAuditLogDetailPath {
                stream_id: CSB_MAIN_STREAM_ID,
                event_id: 999,
            },
            CsbContext::new_test(),
            CsbMainStore::new_for_test(),
            State(state),
            Query(QueryParamState::default()),
        )
        .await;

        assert!(result.is_err());

        Ok(())
    }
}
