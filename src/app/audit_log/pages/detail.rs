use crate::{
    core::ModelLocale,
    finalise::{AllProblems, DocumentData},
    utils::format_hash,
};
use askama::Template;
use axum::{extract::State, response::IntoResponse};

use crate::{
    AppError, AppStore, Context, HtmlTemplate, Overlay, TypstRenderer,
    audit_log::{
        AuditLogDetail, AuditLogPath,
        pages::{AuditLogDetailPath, AuditLogDownloadDocumentsPath},
        structs::FieldChange,
    },
    filters,
};

#[derive(Template)]
#[template(path = "app/audit_log/pages/detail.html")]
struct AuditLogDetailTemplate {
    detail: AuditLogDetail,
    download_path_nl: String,
    download_path_fry: String,
    is_downloadable_state: bool,
    frisian_export_allowed: bool,
    overlay: Overlay,
    hash: String,
}

pub async fn audit_log_detail(
    AuditLogDetailPath { event_id }: AuditLogDetailPath,
    context: Context,
    store: AppStore,
) -> Result<impl IntoResponse, AppError> {
    let events = store.get_events();
    let locale = context.session.locale;

    let detail =
        AuditLogDetail::compute(&events, event_id, locale).ok_or(AppError::GenericNotFound)?;

    let temp_store = create_temp_store(&store, event_id);
    let is_downloadable_state = AllProblems::find_all(&temp_store)?.models_downloadable();

    let hash = store
        .get_events()
        .iter()
        .find(|e| e.event_id == event_id)
        .ok_or(AppError::GenericNotFound)?
        .hash;

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
            overlay: Overlay::default(),
            hash: format_hash(&hash, true),
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
    let temp_store = create_temp_store(&store, event_id);

    if !AllProblems::find_all(&temp_store)?.models_downloadable() {
        return Err(AppError::IncompleteData(
            "Documents cannot be downloaded for this version",
        ));
    }

    let (bundles, filename) = DocumentData::from_store_and_context(&temp_store, &context, locale)?;

    DocumentData::serve_download(
        bundles,
        filename,
        path.to_string(),
        &store,
        &temp_store,
        renderer,
    )
    .await
}

/// Replay the event stream up to and including `event_id` into a throwaway
/// in-memory store, so document state can be inspected as it was back then.
fn create_temp_store(store: &AppStore, event_id: usize) -> AppStore {
    let temp_store = AppStore::new_for_temp_stream(store.election);

    store
        .get_events()
        .iter()
        .take_while(|e| e.event_id <= event_id)
        .for_each(|e| temp_store.apply_event(e.clone()));

    temp_store
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

    #[cfg(feature = "embed-typst")]
    #[tokio::test]
    async fn audit_log_gen_documents_returns_zip_response() -> Result<(), AppError> {
        use axum::{
            http::{StatusCode, header},
            response::IntoResponse,
        };
        use regex::Regex;

        use crate::test_utils::{self, setup_documents_test_state, zip_entry_names};

        let (store, _, context) =
            setup_documents_test_state(1, 5, true, true, crate::ElectionConfig::EK27).await?;
        // the setup does not include political group as an event but directly injects it in the data
        // hence we insert it as an event here
        test_utils::sample_political_group().create(&store).await?;

        // create two remove candidate events
        let mut lists = store.get_candidate_lists();
        let (c1, c2) = (lists[0].candidates[0], lists[0].candidates[1]);
        lists[0].remove_candidate(&store, c1).await?;
        lists[0].remove_candidate(&store, c2).await?;

        let current_event_id = store.current_event_id();

        let response_4_candidates = audit_log_gen_documents(
            AuditLogDownloadDocumentsPath {
                locale: ModelLocale::Nl,
                event_id: current_event_id - 1,
            },
            context.clone(),
            State(TypstRenderer::embedded(
                crate::utils::embed_typst::pdf_context(),
            )),
            store.clone(),
        )
        .await?
        .into_response();

        let response_5_candidates = audit_log_gen_documents(
            AuditLogDownloadDocumentsPath {
                locale: ModelLocale::Nl,
                event_id: current_event_id - 2,
            },
            context,
            State(TypstRenderer::embedded(
                crate::utils::embed_typst::pdf_context(),
            )),
            store.clone(),
        )
        .await?
        .into_response();

        assert_eq!(response_4_candidates.status(), StatusCode::OK);
        let headers = response_4_candidates.headers().clone();
        assert_eq!(
            headers
                .get(header::CONTENT_TYPE)
                .expect("content type header"),
            "application/zip"
        );
        assert!(
            Regex::new("attachment; filename=\"kiesraad-demo-ek27-v\\d+\\.zip\"")
                .unwrap()
                .is_match(
                    headers
                        .get(header::CONTENT_DISPOSITION)
                        .expect("content disposition header")
                        .to_str()
                        .unwrap()
                )
        );
        assert_eq!(
            headers
                .get(header::CACHE_CONTROL)
                .expect("cache control header"),
            "no-store, no-cache, must-revalidate, max-age=0"
        );
        assert_eq!(
            headers.get(header::PRAGMA).expect("pragma header"),
            "no-cache"
        );
        assert_eq!(headers.get(header::EXPIRES).expect("expires header"), "0");

        // check if two download events are present
        let download_event_count = store
            .get_events()
            .iter()
            .filter(|e| matches!(e.payload, crate::AppEvent::DownloadFile { .. }))
            .count();
        assert_eq!(2, download_event_count);

        let h9_count_4_candidates = zip_entry_names(response_4_candidates)
            .await
            .iter()
            .filter(|filename| filename.starts_with("h9-instemmingsverklaringen/"))
            .count();
        assert_eq!(4, h9_count_4_candidates);

        let h9_count_5_candidates = zip_entry_names(response_5_candidates)
            .await
            .iter()
            .filter(|filename| filename.starts_with("h9-instemmingsverklaringen/"))
            .count();
        assert_eq!(5, h9_count_5_candidates);

        Ok(())
    }
}
