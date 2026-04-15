use askama::Template;
use axum::response::IntoResponse;

use crate::{
    AppError, Context, HtmlTemplate, RequestCtx,
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
    ctx: RequestCtx,
) -> Result<impl IntoResponse, AppError> {
    let events = ctx.store.get_events();
    let locale = ctx.locale();

    let detail =
        AuditLogDetail::compute(&events, path.event_id, locale).ok_or(AppError::GenericNotFound)?;

    Ok(HtmlTemplate(AuditLogDetailTemplate { detail }, ctx.context))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppError, AppStore, Context, QueryParamState, RequestCtx,
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
            RequestCtx {
                context: Context::new_test_without_db(),
                store,
                query: QueryParamState::default(),
            },
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
        use crate::test_utils::{sample_candidate_list, sample_list_submitter};

        let store = AppStore::new_for_test();

        let submitter = sample_list_submitter(crate::list_submitters::ListSubmitterId::new());
        let submitter_id = submitter.id;
        let submitter_name = submitter.name.display();
        submitter.create(&store).await?;

        let list_id = crate::candidate_lists::CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.create(&store).await?;

        list.list_submitter_id = Some(submitter_id);
        list.update_submitters(&store).await?;

        let events = store.get_events();
        let target_event_id = events.last().unwrap().event_id;

        let response = audit_log_detail(
            AuditLogDetailPath {
                event_id: target_event_id,
            },
            RequestCtx {
                context: Context::new_test_without_db(),
                store,
                query: QueryParamState::default(),
            },
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        let full_id = submitter_id.to_string();
        assert!(
            body.contains(&format!("<abbr title=\"{full_id}\">")),
            "expected abbreviated link with full id in title; body: {body}"
        );
        assert!(
            body.contains(&format!("/audit-log?search={full_id}")),
            "expected link href to filter audit log by submitter id"
        );
        assert!(
            body.contains(&submitter_name),
            "expected submitter display name to appear next to the abbreviated id"
        );

        Ok(())
    }

    #[tokio::test]
    async fn returns_not_found_for_unknown_event() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let result = audit_log_detail(
            AuditLogDetailPath { event_id: 999 },
            RequestCtx {
                context: Context::new_test_without_db(),
                store,
                query: QueryParamState::default(),
            },
        )
        .await;

        assert!(result.is_err());

        Ok(())
    }
}
