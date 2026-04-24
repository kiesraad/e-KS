use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, AppStore, Context, HtmlTemplate,
    audit_log::{AuditLogDetail, AuditLogPath, pages::AuditLogDetailPath, structs::FieldChange},
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
    async fn renders_entity_ref_link_and_description_for_id_diff() -> Result<(), AppError> {
        use crate::test_utils::{sample_candidate_list, sample_person_with_last_name};

        let store = AppStore::new_for_test();

        let person = sample_person_with_last_name(PersonId::new(), "Janssen");
        let person_id = person.id;
        let person_name = person.name.display();
        person.create(&store).await?;

        let list_id = crate::candidate_lists::CandidateListId::new();
        let list = sample_candidate_list(list_id);
        list.create(&store).await?;

        let mut list = store.get_candidate_list(list_id)?;
        list.append_candidate(&store, person_id).await?;

        let events = store.get_events();
        let target_event_id = events.last().unwrap().event_id;

        let response = audit_log_detail(
            AuditLogDetailPath {
                event_id: target_event_id,
            },
            Context::new_test_without_db(),
            store,
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        let full_id = person_id.to_string();
        assert!(
            body.contains(&format!("<abbr title=\"{full_id}\">")),
            "expected abbreviated link with full id in title; body: {body}"
        );
        assert!(
            body.contains(&format!("/audit-log?search={full_id}")),
            "expected link href to filter audit log by person id"
        );
        assert!(
            body.contains(&person_name),
            "expected person display name to appear next to the abbreviated id"
        );

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
