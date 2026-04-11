use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    audit_log::{AuditLogDetail, AuditLogPath, pages::AuditLogDetailPath},
    filters,
};

#[derive(Template)]
#[template(path = "audit_log/pages/detail.html")]
struct AuditLogDetailTemplate {
    detail: AuditLogDetail,
}

pub async fn audit_log_detail(
    path: AuditLogDetailPath,
    context: Context,
    store: AppStore,
) -> Result<impl IntoResponse, AppError> {
    let events = store.get_events();
    let locale = context.session.locale;

    let detail =
        AuditLogDetail::compute(&events, path.event_id, locale).ok_or(AppError::GenericNotFound)?;

    Ok(HtmlTemplate(AuditLogDetailTemplate { detail }, context))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context,
        persons::PersonId,
        test_utils::{response_body_string, sample_person},
    };
    use axum::{http::StatusCode, response::IntoResponse};

    #[tokio::test]
    async fn renders_detail_for_create_event() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let person = sample_person(PersonId::new());
        person.create(&store).await?;

        let response = audit_log_detail(
            AuditLogDetailPath { event_id: 1 },
            Context::new_test_without_db(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Created person"));
        assert!(body.contains("diff-table"));

        Ok(())
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_event() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let result = audit_log_detail(
            AuditLogDetailPath { event_id: 999 },
            Context::new_test_without_db(),
            store,
        )
        .await;

        assert!(result.is_err());

        Ok(())
    }
}
