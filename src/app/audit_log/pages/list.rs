use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    audit_log::{AuditLogEntry, pages::AuditLogPath},
    filters,
    pagination::Pagination,
};

const PER_PAGE: usize = 20;

#[derive(Template)]
#[template(path = "audit_log/pages/list.html")]
struct AuditLogTemplate {
    entries: Vec<AuditLogEntry>,
    pagination: crate::pagination::PaginationInfo<NoSort>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NoSort;

pub async fn audit_log(
    _: AuditLogPath,
    context: Context,
    store: AppStore,
    pagination: Pagination<NoSort>,
) -> Result<impl IntoResponse, AppError> {
    let all_events = store.get_events();
    let total = all_events.len();

    let pagination = Pagination {
        per_page: PER_PAGE,
        ..pagination
    }
    .set_total(total);

    let locale = context.session.locale;
    let entries: Vec<AuditLogEntry> = all_events
        .into_iter()
        .rev()
        .skip(pagination.offset())
        .take(pagination.limit())
        .map(|event| AuditLogEntry::new(event, locale))
        .collect();

    Ok(HtmlTemplate(
        AuditLogTemplate {
            entries,
            pagination,
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
    use axum::{http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn renders_empty_audit_log() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let response = audit_log(
            AuditLogPath {},
            Context::new_test_without_db(),
            store,
            Pagination::default(),
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
}
