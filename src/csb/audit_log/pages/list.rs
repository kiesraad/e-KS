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

const MAIN_STREAM_VALUE: &str = "main";

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
    /// - [`MAIN_STREAM_VALUE`] to show only the global stream
    /// - a UUID string to show only that import stream
    /// - `None` to show all streams combined
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

    // Add main stream events
    if active_stream.is_none() || active_stream == Some(MAIN_STREAM_VALUE) {
        let main_stream_id = main_store.stream_id;
        for event in main_store.data.read().events.iter() {
            all_entries.push(CsbAuditLogEntry::from_main_event(
                event.clone(),
                main_stream_id,
                locale,
            ));
        }
    }

    // Add import stream events
    if active_stream.is_none() || active_stream.is_some_and(|s| s != MAIN_STREAM_VALUE) {
        for store in &import_stores {
            let stream_id_str = store.stream_id.to_string();
            if let Some(s) = active_stream
                && s != MAIN_STREAM_VALUE
                && s != stream_id_str.as_str()
            {
                continue;
            }

            let label = import_stream_labels
                .iter()
                .find(|(id, _)| *id == store.stream_id)
                .map(|(_, l)| l.clone())
                .unwrap_or_default();

            for event in store.data.read().events.iter() {
                all_entries.push(CsbAuditLogEntry::from_import_event(
                    event.clone(),
                    store.stream_id,
                    label.clone(),
                    locale,
                ));
            }
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
