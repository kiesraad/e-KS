use askama::Template;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};

use crate::{
    AppError, AppState, Context, CsbContext, CsbMainStore, HtmlTemplate, StreamId,
    csb::audit_log::{pages::CsbAuditLogPath, structs::CsbAuditLogEntry},
    filters,
    pagination::Pagination,
};

const PER_PAGE: usize = 20;

pub struct EventTypeCategory {
    pub key: &'static str,
    pub event_types: &'static [&'static str],
}

/// Event type categories grouped with their specific event keys, used by the
/// filter dropdown to render <optgroup>s with fine-grained <option>s.
///
/// Category label translations (referenced dynamically in the template):
/// trans!("audit_log.filter.category.import", _)
/// trans!("audit_log.filter.category.omission", _)
/// trans!("audit_log.filter.category.system", _)
pub const EVENT_TYPES_BY_CATEGORY: &[EventTypeCategory] = &[
    EventTypeCategory {
        key: "import",
        event_types: &["import"],
    },
    EventTypeCategory {
        key: "omission",
        event_types: &["create_omission", "update_omission", "delete_omission"],
    },
    EventTypeCategory {
        key: "system",
        event_types: &["developer_login"],
    },
];

/// Filters for the CSB audit log list view.
#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct CsbAuditLogFilter {
    /// The stream can be:
    /// - `None` to show the CSB main stream
    /// - a UUID string to show that import stream
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

impl CsbAuditLogFilter {
    pub fn as_query_suffix(&self) -> String {
        match serde_urlencoded::to_string(self) {
            Ok(query) if !query.is_empty() => format!("&{query}"),
            _ => String::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.stream.as_deref().is_some_and(|s| !s.is_empty())
            || self.event_type.as_deref().is_some_and(|s| !s.is_empty())
            || self.search.as_deref().is_some_and(|s| !s.is_empty())
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NoSort;

#[derive(Template)]
#[template(path = "csb/audit_log/pages/list.html")]
struct CsbAuditLogTemplate {
    entries: Vec<CsbAuditLogEntry>,
    pagination: crate::pagination::PaginationInfo<NoSort>,
    filter: CsbAuditLogFilter,
    /// Import streams available for filtering: (stream_id, label).
    import_streams: Vec<(StreamId, String)>,
    event_types_by_category: &'static [EventTypeCategory],
}

pub async fn csb_audit_log(
    _: CsbAuditLogPath,
    context: CsbContext,
    main_store: CsbMainStore,
    State(state): State<AppState>,
    pagination: Pagination<NoSort>,
    Query(filter): Query<CsbAuditLogFilter>,
) -> Result<impl IntoResponse, AppError> {
    let locale = context.session.locale;
    let import_stores = state.csb_store_registry.stores_by_scope().await?;

    // Build a short label for each import stream from its import event
    let import_stream_labels: Vec<(StreamId, String)> = import_stores
        .iter()
        .map(|store| {
            let name = store.get_political_group().csb_display_name();
            (store.stream_id, name)
        })
        .collect();

    let active_stream = filter.stream.as_deref().filter(|s| !s.is_empty());
    let active_event_type = filter.event_type.as_deref().filter(|s| !s.is_empty());
    let active_search = filter.search.as_deref().filter(|s| !s.is_empty());

    let mut all_entries: Vec<CsbAuditLogEntry> = Vec::new();

    if let Some(stream_id) = active_stream {
        // Add import stream events
        let store = import_stores
            .iter()
            .find(|s| s.stream_id.to_string() == stream_id)
            .ok_or(AppError::GenericNotFound)?;

        let label = store.get_political_group().csb_display_name();

        for event in store.data.read().events.iter() {
            all_entries.push(CsbAuditLogEntry::from_event(
                event.clone(),
                store.stream_id,
                label.clone(),
                locale,
            ));
        }
    } else {
        // Add main stream events
        for event in main_store.data.read().events.iter() {
            all_entries.push(CsbAuditLogEntry::from_main_event(
                event.clone(),
                main_store.stream_id,
                locale,
            ));
        }
    }

    // Apply event type filter.
    if let Some(et) = active_event_type {
        all_entries.retain(|e| e.event_category == et || e.event_key == et);
    }

    // Apply search filter.
    if let Some(q) = active_search {
        all_entries.retain(|e| e.matches_search(q));
    }

    // Newest first.
    all_entries.sort_by_key(|b| std::cmp::Reverse(b.created_at));

    let total = all_entries.len();
    let pagination = Pagination {
        per_page: PER_PAGE,
        ..pagination
    }
    .set_total(total);

    let entries = all_entries
        .into_iter()
        .skip(pagination.offset())
        .take(pagination.limit())
        .collect();

    Ok(HtmlTemplate(
        CsbAuditLogTemplate {
            entries,
            pagination,
            filter,
            import_streams: import_stream_labels,
            event_types_by_category: EVENT_TYPES_BY_CATEGORY,
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
        StreamId,
        csb::{
            CSB_MAIN_STREAM_ID, Omission, audit_log::pages::CsbAuditLogPath,
            omission::OmissionCategory,
        },
        pagination::Pagination,
        test_utils::response_body_string,
    };

    fn no_filter() -> Query<CsbAuditLogFilter> {
        Query(CsbAuditLogFilter::default())
    }

    async fn call(
        main_store: CsbMainStore,
        state: AppState,
        filter: Query<CsbAuditLogFilter>,
    ) -> Result<axum::response::Response, AppError> {
        Ok(csb_audit_log(
            CsbAuditLogPath,
            CsbContext::new_test(),
            main_store,
            State(state),
            Pagination::default(),
            filter,
        )
        .await?
        .into_response())
    }

    #[tokio::test]
    async fn renders_empty_audit_log() -> Result<(), AppError> {
        let response = call(
            CsbMainStore::new_for_test(),
            AppState::new_for_tests().await,
            no_filter(),
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Audit log"));
        // Should show empty message, not a table
        assert!(!body.contains("<table"));

        Ok(())
    }

    #[tokio::test]
    async fn renders_main_stream_events() -> Result<(), AppError> {
        let main_store = CsbMainStore::new_for_test();
        main_store
            .update(CsbMainEvent::DeveloperLogin {
                stream_id: CSB_MAIN_STREAM_ID,
            })
            .await?;

        let response = call(main_store, AppState::new_for_tests().await, no_filter()).await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("<table"));
        assert!(body.contains("<td>Developer login</td>"));
        assert!(body.contains("<td>Main CSB stream</td>"));

        Ok(())
    }

    #[tokio::test]
    async fn renders_import_stream_events() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let import_stream_id = StreamId::new();
        let csb_store = state
            .csb_store_for_stream(import_stream_id, ElectionConfig::EK27)
            .await?;
        csb_store.update(CsbEvent::SetFinished(true)).await?;

        let response = call(
            CsbMainStore::new_for_test(),
            state,
            Query(CsbAuditLogFilter {
                stream: Some(import_stream_id.to_string()),
                ..Default::default()
            }),
        )
        .await?;

        let body = response_body_string(response).await;
        assert!(body.contains("<td>Set finished state</td>"));

        Ok(())
    }

    #[tokio::test]
    async fn shows_events_newest_first() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let import_stream_id = StreamId::new();
        let csb_store = state
            .csb_store_for_stream(import_stream_id, ElectionConfig::EK27)
            .await?;
        csb_store.update(CsbEvent::SetFinished(true)).await?;
        csb_store
            .update(CsbEvent::CreateOmission(Omission::new(
                OmissionCategory::General,
                "test".to_string(),
                "test".to_string(),
            )))
            .await?;

        let response = call(
            CsbMainStore::new_for_test(),
            state,
            Query(CsbAuditLogFilter {
                stream: Some(import_stream_id.to_string()),
                ..Default::default()
            }),
        )
        .await?;

        let body = response_body_string(response).await;
        let finished_pos = body
            .find("<td>Set finished state</td>")
            .expect("set finished event");
        let omission_pos = body
            .find("<td>Created omission</td>")
            .expect("create omission event");
        assert!(
            omission_pos < finished_pos,
            "newer event (create omission) should appear before older event (set finished)"
        );

        Ok(())
    }

    #[tokio::test]
    async fn paginates_results() -> Result<(), AppError> {
        let main_store = CsbMainStore::new_for_test();
        for _ in 0..PER_PAGE + 5 {
            main_store
                .update(CsbMainEvent::DeveloperLogin {
                    stream_id: CSB_MAIN_STREAM_ID,
                })
                .await?;
        }

        let response = call(main_store, AppState::new_for_tests().await, no_filter()).await?;

        let body = response_body_string(response).await;
        assert!(body.contains("Pagination"));
        let row_count = body.matches("<td>Developer login</td>").count();
        assert_eq!(row_count, PER_PAGE);

        Ok(())
    }

    #[tokio::test]
    async fn filters_by_main_stream() -> Result<(), AppError> {
        let main_store = CsbMainStore::new_for_test();
        main_store
            .update(CsbMainEvent::DeveloperLogin {
                stream_id: CSB_MAIN_STREAM_ID,
            })
            .await?;

        let state = AppState::new_for_tests().await;
        let csb_store = state
            .csb_store_for_stream(StreamId::new(), ElectionConfig::EK27)
            .await?;
        csb_store.update(CsbEvent::SetFinished(true)).await?;

        let response = call(main_store, state, no_filter()).await?;

        let body = response_body_string(response).await;
        assert!(body.contains("<td>Developer login</td>"));
        assert!(!body.contains("<td>Set finished state</td>"));

        Ok(())
    }

    #[tokio::test]
    async fn filters_by_import_stream() -> Result<(), AppError> {
        let main_store = CsbMainStore::new_for_test();
        main_store
            .update(CsbMainEvent::DeveloperLogin {
                stream_id: CSB_MAIN_STREAM_ID,
            })
            .await?;

        let state = AppState::new_for_tests().await;
        let import_stream_id = StreamId::new();
        let csb_store = state
            .csb_store_for_stream(import_stream_id, ElectionConfig::EK27)
            .await?;
        csb_store.update(CsbEvent::SetFinished(true)).await?;

        let response = call(
            main_store,
            state,
            Query(CsbAuditLogFilter {
                stream: Some(import_stream_id.to_string()),
                ..Default::default()
            }),
        )
        .await?;

        let body = response_body_string(response).await;
        assert!(!body.contains("<td>Developer login</td>"));
        assert!(body.contains("<td>Set finished state</td>"));

        Ok(())
    }

    #[tokio::test]
    async fn filters_by_event_type() -> Result<(), AppError> {
        let main_store = CsbMainStore::new_for_test();
        main_store
            .update(CsbMainEvent::DeveloperLogin {
                stream_id: CSB_MAIN_STREAM_ID,
            })
            .await?;

        let state = AppState::new_for_tests().await;
        let import_stream_id = StreamId::new();
        let csb_store = state
            .csb_store_for_stream(import_stream_id, ElectionConfig::EK27)
            .await?;
        csb_store.update(CsbEvent::SetFinished(true)).await?;

        let response = call(
            main_store,
            state,
            Query(CsbAuditLogFilter {
                stream: Some(import_stream_id.to_string()),
                event_type: Some("set_finished".to_string()),
                ..Default::default()
            }),
        )
        .await?;

        let body = response_body_string(response).await;
        assert!(body.contains("<td>Set finished state</td>"));
        assert!(!body.contains("<td>Developer login</td>"));

        Ok(())
    }

    #[tokio::test]
    async fn searches_by_description() -> Result<(), AppError> {
        let main_store = CsbMainStore::new_for_test();
        main_store
            .update(CsbMainEvent::DeveloperLogin {
                stream_id: CSB_MAIN_STREAM_ID,
            })
            .await?;

        let state = AppState::new_for_tests().await;
        let csb_store = state
            .csb_store_for_stream(StreamId::new(), ElectionConfig::EK27)
            .await?;
        csb_store.update(CsbEvent::SetFinished(true)).await?;

        let response = call(
            main_store,
            state,
            Query(CsbAuditLogFilter {
                search: Some("Developer".to_string()),
                ..Default::default()
            }),
        )
        .await?;

        let body = response_body_string(response).await;
        assert!(body.contains("<td>Developer login</td>"));
        assert!(!body.contains("<td>Set finished state</td>"));

        Ok(())
    }

    #[tokio::test]
    async fn reset_button_only_shown_when_filter_active() -> Result<(), AppError> {
        let main_store = CsbMainStore::new_for_test();
        main_store
            .update(CsbMainEvent::DeveloperLogin {
                stream_id: CSB_MAIN_STREAM_ID,
            })
            .await?;

        let state = AppState::new_for_tests().await;

        let response = call(main_store.clone(), state.clone(), no_filter()).await?;
        let body = response_body_string(response).await;
        assert!(!body.contains("/csb/audit-log\" class=\"button secondary\">"));

        let response = call(
            main_store,
            state,
            Query(CsbAuditLogFilter {
                event_type: Some("system".to_string()),
                ..Default::default()
            }),
        )
        .await?;
        let body = response_body_string(response).await;
        assert!(body.contains("/csb/audit-log\" class=\"button secondary\">"));

        Ok(())
    }
}
