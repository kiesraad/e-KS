use crate::{
    AppEvent,
    core::ModelLocale,
    submit::{DocumentData, Problems, ZIP_CONTENT_TYPE},
};
use askama::Template;
use axum::{extract::State, response::IntoResponse};

use crate::{
    AppError, AppStore, Context, HtmlTemplate, TypstRenderer,
    audit_log::{
        AuditLogDetail, AuditLogPath,
        pages::{AuditLogDetailPath, AuditLogDownloadDocumentsPath},
        structs::FieldChange,
    },
    filters,
};

#[derive(Template)]
#[template(path = "audit_log/pages/detail.html")]
struct AuditLogDetailTemplate {
    detail: AuditLogDetail,
    download_path_nl: String,
    download_path_fry: String,
    is_downloadable_state: bool,
    frisian_export_allowed: bool,
}

pub async fn audit_log_detail(
    AuditLogDetailPath { event_id }: AuditLogDetailPath,
    context: Context,
    store: AppStore,
) -> Result<impl IntoResponse, AppError> {
    let events = store.get_events();
    let locale = context.session.locale;
    let temp_store = AppStore::new_for_temp_stream(store.election).await;

    store
        .get_events()
        .iter()
        .take_while(|e| e.event_id <= event_id)
        .zip(1..)
        .for_each(|(e, i)| temp_store.apply_event(i, e.clone()));

    let is_downloadable_state = Problems::find_all(&temp_store).models_downloadable();

    let detail =
        AuditLogDetail::compute(&events, event_id, locale).ok_or(AppError::GenericNotFound)?;

    Ok(HtmlTemplate(
        AuditLogDetailTemplate {
            detail,
            download_path_nl: AuditLogDownloadDocumentsPath {
                event_id,
                locale: ModelLocale::Nl,
            }
            .to_string(),
            download_path_fry: AuditLogDownloadDocumentsPath {
                event_id,
                locale: ModelLocale::Fry,
            }
            .to_string(),
            is_downloadable_state,
            frisian_export_allowed: context.election.frisian_export_allowed(),
        },
        context,
    ))
}

pub async fn audit_log_gen_documents(
    path @ AuditLogDownloadDocumentsPath { event_id, locale }: AuditLogDownloadDocumentsPath,
    context: Context,
    State(renderer): State<TypstRenderer>,
    store: AppStore,
) -> Result<impl IntoResponse, AppError> {
    let temp_store = AppStore::new_for_temp_stream(store.election).await;

    store
        .get_events()
        .iter()
        .take_while(|e| e.event_id <= event_id)
        .zip(1..)
        .for_each(|(e, i)| temp_store.apply_event(i, e.clone()));

    let bundles = DocumentData::from_store_and_context(&temp_store, &context, locale).await?;

    let Some(document_data) = bundles.first() else {
        return Err(AppError::IncompleteData("No candidate lists"));
    };

    let filename = document_data.archive_filename();

    tracing::info!(
        filename,
        content_type = ZIP_CONTENT_TYPE,
        lists = temp_store.get_candidate_list_count(),
        "file download served",
    );

    store
        .update(AppEvent::DownloadFile {
            file_name: filename.clone(),
            download_path: path.to_string(),
        })
        .await?;

    DocumentData::to_zip_response(bundles, filename, renderer).await
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
