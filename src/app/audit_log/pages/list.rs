use askama::Template;
use axum::{extract::Query, response::IntoResponse};

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    audit_log::{AuditLogEntry, pages::AuditLogPath},
    filters,
    pagination::Pagination,
};

const PER_PAGE: usize = 20;

/// Event type categories for the filter dropdown.
///
/// Category label translations (referenced dynamically in the template):
/// trans!("audit_log.filter.category.political_group", _)
/// trans!("audit_log.filter.category.person", _)
/// trans!("audit_log.filter.category.candidate_list", _)
/// trans!("audit_log.filter.category.authorised_agent", _)
/// trans!("audit_log.filter.category.list_submitter", _)
/// trans!("audit_log.filter.category.substitute_submitter", _)
/// trans!("audit_log.filter.category.system", _)
pub const EVENT_CATEGORIES: &[&str] = &[
    "political_group",
    "person",
    "candidate_list",
    "authorised_agent",
    "list_submitter",
    "substitute_submitter",
    "system",
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
}

#[derive(Template)]
#[template(path = "audit_log/pages/list.html")]
struct AuditLogTemplate {
    entries: Vec<AuditLogEntry>,
    pagination: crate::pagination::PaginationInfo<NoSort>,
    filter: AuditLogFilter,
    event_categories: &'static [&'static str],
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
                .map(|et| event.payload.event_category() == et)
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
            event_categories: EVENT_CATEGORIES,
        },
        context,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, PoliticalGroupId,
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
        assert!(body.contains("Created person"));
        assert!(body.contains(&person.name.display()));

        Ok(())
    }

    #[tokio::test]
    async fn shows_events_in_reverse_chronological_order() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let pg = sample_political_group(PoliticalGroupId::new());
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
        let person_pos = body.find("Created person").expect("person event");
        let pg_pos = body.find("Updated political group").expect("pg event");
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
        let row_count = body.matches("Created person").count();
        assert_eq!(row_count, PER_PAGE);

        Ok(())
    }

    #[tokio::test]
    async fn filters_by_event_type() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let pg = sample_political_group(PoliticalGroupId::new());
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
        assert!(body.contains("Created person"));
        assert!(!body.contains("Updated political group"));

        Ok(())
    }

    #[tokio::test]
    async fn searches_by_details() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let person = sample_person(PersonId::new());
        let name = person.name.display();
        person.create(&store).await?;
        let pg = sample_political_group(PoliticalGroupId::new());
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
        assert!(body.contains("Created person"));
        assert!(!body.contains("Updated political group"));

        Ok(())
    }
}
