use askama::Template;
use axum::{extract::Query, response::IntoResponse};

use crate::{
    AppError, AppStore, Context, Event, HtmlTemplate,
    audit_log::{AuditLogEntry, pages::AuditLogPath},
    filters,
    pagination::Pagination,
};

const PER_PAGE: usize = 20;

pub struct EventTypeCategory {
    pub key: &'static str,
    pub event_types: &'static [&'static str],
}

/// Event type categories grouped with their specific event keys, used by the
/// filter dropdown to render `<optgroup>`s with fine-grained `<option>`s.
///
/// Category label translations (referenced dynamically in the template):
/// trans!("audit_log.filter.category.political_group", _)
/// trans!("audit_log.filter.category.person", _)
/// trans!("audit_log.filter.category.candidate_list", _)
/// trans!("audit_log.filter.category.name_authorisation", _)
/// trans!("audit_log.filter.category.list_submitter", _)
/// trans!("audit_log.filter.category.substitute_submitter", _)
/// trans!("audit_log.filter.category.system", _)
pub const EVENT_TYPES_BY_CATEGORY: &[EventTypeCategory] = &[
    EventTypeCategory {
        key: "political_group",
        event_types: &["update_political_group"],
    },
    EventTypeCategory {
        key: "person",
        event_types: &[
            "create_person",
            "update_person",
            "update_person_address",
            "update_person_representative",
            "delete_person",
        ],
    },
    EventTypeCategory {
        key: "candidate_list",
        event_types: &[
            "create_candidate_list",
            "update_candidate_list_districts",
            "update_candidate_list_order",
            "add_candidate_to_list",
            "remove_candidate_from_list",
            "delete_candidate_list",
        ],
    },
    EventTypeCategory {
        key: "name_authorisation",
        event_types: &[
            "create_name_authorisation",
            "update_name_authorisation",
            "delete_name_authorisation",
        ],
    },
    EventTypeCategory {
        key: "list_submitter",
        event_types: &["update_list_submitter"],
    },
    EventTypeCategory {
        key: "substitute_submitter",
        event_types: &[
            "create_substitute_submitter",
            "update_substitute_submitter",
            "delete_substitute_submitter",
        ],
    },
    EventTypeCategory {
        key: "system",
        event_types: &[
            "developer_login",
            "download_file",
            "hide_download_warning",
            "export_csv",
            "import_csv",
        ],
    },
];

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct AuditLogFilter {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

impl AuditLogFilter {
    /// Build a query string fragment (with leading `&`) for the active filters.
    pub fn as_query_suffix(&self) -> String {
        match serde_urlencoded::to_string(self) {
            Ok(query) if !query.is_empty() => format!("&{query}"),
            _ => String::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.event_type.as_deref().is_some_and(|s| !s.is_empty())
            || self.search.as_deref().is_some_and(|s| !s.is_empty())
    }
}

#[derive(Template)]
#[template(path = "app/audit_log/pages/list.html")]
struct AuditLogTemplate {
    entries: Vec<AuditLogEntry>,
    pagination: crate::pagination::PaginationInfo<NoSort>,
    filter: AuditLogFilter,
    event_types_by_category: &'static [EventTypeCategory],
}

#[derive(Debug, Default, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NoSort;

pub async fn audit_log(
    _: AuditLogPath,
    context: Context,
    store: AppStore,
    pagination: Pagination<NoSort>,
    Query(filter): Query<AuditLogFilter>,
) -> Result<impl IntoResponse, AppError> {
    let locale = context.session.locale;

    let active_event_type = filter.event_type.as_deref().filter(|s| !s.is_empty());
    let active_search = filter.search.as_deref().filter(|s| !s.is_empty());

    // Convert all events to entries, applying event_type filter early
    let all_entries: Vec<AuditLogEntry> = store
        .get_events()
        .into_iter()
        .rev()
        .filter(|event| {
            active_event_type
                .map(|et| event.payload.category() == et || event.payload.key() == et)
                .unwrap_or(true)
        })
        .map(|event| AuditLogEntry::new(event, locale))
        .filter(|entry| {
            active_search
                .map(|q| entry.matches_search(q))
                .unwrap_or(true)
        })
        .collect();

    let total = all_entries.len();

    let pagination = Pagination {
        per_page: PER_PAGE,
        ..pagination
    }
    .set_total(total);

    let entries: Vec<AuditLogEntry> = all_entries
        .into_iter()
        .skip(pagination.offset())
        .take(pagination.limit())
        .collect();

    Ok(HtmlTemplate(
        AuditLogTemplate {
            entries,
            pagination,
            filter,
            event_types_by_category: EVENT_TYPES_BY_CATEGORY,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context,
        pagination::Pagination,
        persons::PersonId,
        test_utils::{response_body_string, sample_person, sample_political_group},
    };
    use axum::{extract::Query, http::StatusCode, response::IntoResponse};

    fn no_filter() -> Query<AuditLogFilter> {
        Query(AuditLogFilter::default())
    }

    #[tokio::test]
    async fn renders_empty_audit_log() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let response = audit_log(
            AuditLogPath {},
            Context::new_test_without_db(),
            store,
            Pagination::default(),
            no_filter(),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Audit log"));
        // Should show empty message, not a table
        assert!(!body.contains("<table"));

        Ok(())
    }

    #[tokio::test]
    async fn renders_events_in_table() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let person = sample_person(PersonId::new());
        person.create(&store).await?;

        let response = audit_log(
            AuditLogPath {},
            Context::new_test_without_db(),
            store,
            Pagination::default(),
            no_filter(),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("<table"));
        assert!(body.contains("<td>Created person</td>"));
        assert!(body.contains(&person.name.display()));

        Ok(())
    }

    #[tokio::test]
    async fn shows_events_in_reverse_chronological_order() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let pg = sample_political_group();
        pg.update(&store).await?;
        let person = sample_person(PersonId::new());
        person.create(&store).await?;

        let response = audit_log(
            AuditLogPath {},
            Context::new_test_without_db(),
            store,
            Pagination::default(),
            no_filter(),
        )
        .await
        .unwrap()
        .into_response();

        let body = response_body_string(response).await;
        let person_pos = body.find("<td>Created person</td>").expect("person event");
        let pg_pos = body
            .find("<td>Updated political group</td>")
            .expect("pg event");
        assert!(
            person_pos < pg_pos,
            "newest event (create person) should appear before older event (update pg)"
        );

        Ok(())
    }

    #[tokio::test]
    async fn paginates_results() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        // Create more events than PER_PAGE
        for _ in 0..PER_PAGE + 5 {
            let person = sample_person(PersonId::new());
            person.create(&store).await?;
        }

        let response = audit_log(
            AuditLogPath {},
            Context::new_test_without_db(),
            store,
            Pagination::default(),
            no_filter(),
        )
        .await
        .unwrap()
        .into_response();

        let body = response_body_string(response).await;
        // Should show pagination controls
        assert!(body.contains("Pagination"));
        // Count table rows - should be PER_PAGE
        let row_count = body.matches("<td>Created person</td>").count();
        assert_eq!(row_count, PER_PAGE);

        Ok(())
    }

    #[tokio::test]
    async fn filters_by_event_type() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let pg = sample_political_group();
        pg.update(&store).await?;
        let person = sample_person(PersonId::new());
        person.create(&store).await?;

        let response = audit_log(
            AuditLogPath {},
            Context::new_test_without_db(),
            store,
            Pagination::default(),
            Query(AuditLogFilter {
                event_type: Some("person".to_string()),
                search: None,
            }),
        )
        .await
        .unwrap()
        .into_response();

        let body = response_body_string(response).await;
        assert!(body.contains("<td>Created person</td>"));
        assert!(!body.contains("<td>Updated political group</td>"));

        Ok(())
    }

    #[tokio::test]
    async fn filters_by_specific_event_key() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let person = sample_person(PersonId::new());
        person.create(&store).await?;
        let other = sample_person(PersonId::new());
        other.create(&store).await?;
        other.delete(&store).await?;

        let response = audit_log(
            AuditLogPath {},
            Context::new_test_without_db(),
            store,
            Pagination::default(),
            Query(AuditLogFilter {
                event_type: Some("delete_person".to_string()),
                search: None,
            }),
        )
        .await
        .unwrap()
        .into_response();

        let body = response_body_string(response).await;
        assert!(body.contains("<td>Deleted person</td>"));
        assert!(!body.contains("<td>Created person</td>"));

        Ok(())
    }

    #[tokio::test]
    async fn reset_button_only_shown_when_filter_active() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let person = sample_person(PersonId::new());
        person.create(&store).await?;

        let response = audit_log(
            AuditLogPath {},
            Context::new_test_without_db(),
            store.clone(),
            Pagination::default(),
            no_filter(),
        )
        .await
        .unwrap()
        .into_response();

        let body = response_body_string(response).await;
        assert!(!body.contains("/audit-log\" class=\"button secondary\">Reset"));

        let response = audit_log(
            AuditLogPath {},
            Context::new_test_without_db(),
            store,
            Pagination::default(),
            Query(AuditLogFilter {
                event_type: Some("person".to_string()),
                search: None,
            }),
        )
        .await
        .unwrap()
        .into_response();

        let body = response_body_string(response).await;
        assert!(body.contains("/audit-log\" class=\"button secondary\">Reset"));

        Ok(())
    }

    #[tokio::test]
    async fn searches_by_details() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let person = sample_person(PersonId::new());
        let name = person.name.display();
        person.create(&store).await?;
        let pg = sample_political_group();
        pg.update(&store).await?;

        let response = audit_log(
            AuditLogPath {},
            Context::new_test_without_db(),
            store,
            Pagination::default(),
            Query(AuditLogFilter {
                event_type: None,
                search: Some(name.clone()),
            }),
        )
        .await
        .unwrap()
        .into_response();

        let body = response_body_string(response).await;
        assert!(body.contains("<td>Created person</td>"));
        assert!(!body.contains("<td>Updated political group</td>"));

        Ok(())
    }
}
