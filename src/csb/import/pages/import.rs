use std::time::Duration;

use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::{
    AppError, AppRequestState, Context, CsbAction, CsbContext, CsbStore, CsbUser, Form,
    HtmlTemplate, Locale, PgStoreData, StreamId,
    csb::examination::{CsbExaminationOverviewPath, CsbPoliticalGroupPath},
    filters,
    projection::WithCorrections,
    redirect_success,
    structs::{
        brp::{BrpClient, BrpStatus},
        persons::Person,
    },
    trans,
    utils::parse_hash_prefix,
};

use super::{CsbCreateEmptyPath, CsbImportPath};

const BRP_COURTESY_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Template)]
#[template(path = "csb/import/pages/import.html")]
struct CsbImportTemplate {
    hash: String,
    error: Option<String>,
    warning: Option<String>,
}

fn render_import(
    context: CsbContext,
    hash: String,
    error: Option<String>,
    warning: Option<String>,
) -> Response {
    HtmlTemplate(
        CsbImportTemplate {
            hash,
            error,
            warning,
        },
        context,
    )
    .into_response()
}

/// Render the placeholder import page.
pub async fn import(_: CsbImportPath, context: CsbContext) -> Result<Response, AppError> {
    Ok(render_import(context, String::new(), None, None))
}

/// Form payload for the import page: the chain hash of the package to import,
/// and the hash for which a duplicate-import warning was already confirmed.
#[derive(Debug, Deserialize)]
pub struct ImportForm {
    pub hash: String,
    pub confirmed_hash: Option<String>,
}

/// Outcome of an import attempt that did not fail outright.
enum ImportOutcome {
    Imported(Response),
    /// The source stream was already imported into a live CSB store carrying
    /// this appellation; the user must confirm before importing again.
    AlreadyImported {
        appellation: String,
    },
}

/// Import the package identified by the submitted chain hash.
pub async fn import_submit<S: AppRequestState>(
    _: CsbImportPath,
    State(state): State<S>,
    context: CsbContext,
    Form(form): Form<ImportForm>,
) -> Result<Response, AppError> {
    let hash = form.hash.clone();
    let locale = context.session.locale;
    let user = context.user()?;
    match do_import(&state, form, user, locale).await {
        Ok(ImportOutcome::Imported(response)) => Ok(response),
        Ok(ImportOutcome::AlreadyImported { appellation }) => Ok(render_import(
            context,
            hash,
            None,
            Some(trans!(
                "csb.import.warning.already_imported",
                locale,
                appellation
            )),
        )),
        Err(AppError::UserError(msg)) => Ok(render_import(context, hash, Some(msg), None)),
        Err(AppError::AmbiguousHash) => Ok(render_import(
            context,
            hash,
            Some(trans!("csb.import.error.ambiguous_hash", locale)),
            None,
        )),
        Err(e) => Err(e),
    }
}

/// Locates the political-group event whose hash matches the entry, replays that
/// stream up to the event into an [`PgStoreData`] snapshot (its event log
/// excluded), and records the snapshot in a [`CsbAction::Import`] persisted under
/// a fresh CSB stream keyed on the source election. The source `stream_id` is
/// carried on the event for reference; it is never reused as the CSB partition,
/// which would collide with the PG stream's own events there.
async fn do_import<S: AppRequestState>(
    state: &S,
    form: ImportForm,
    user: CsbUser,
    locale: Locale,
) -> Result<ImportOutcome, AppError> {
    let hash_prefix = parse_hash_prefix(&form.hash)
        .ok_or_else(|| AppError::UserError(trans!("csb.import.error.invalid_hash", locale)))?;

    let (source_stream_id, source_election, event_id) = state
        .store_registry()
        .persistence()
        .find_event_by_hash_prefix(&hash_prefix)
        .await?
        .ok_or_else(|| AppError::UserError(trans!("csb.import.error.not_found", locale)))?;

    // Importing a source stream that was already imported is allowed, but only
    // after the user confirms the warning for this exact hash entry.
    let confirmed = form.confirmed_hash.as_deref() == Some(form.hash.as_str());
    if !confirmed {
        for store in state.csb_store_registry().stores_by_scope().await? {
            let already_imported_and_not_deleted =
                store.data.read().events.first().is_some_and(|e| {
                    matches!(&e.payload.action, CsbAction::Import { source_stream_id: sid, .. } if *sid == source_stream_id) &&
                    !store.is_deleted()
                });
            if already_imported_and_not_deleted {
                let appellation = store.get_appellation(WithCorrections::All);
                return Ok(ImportOutcome::AlreadyImported { appellation });
            }
        }
    }

    // Replay the source stream up to the matched event into a snapshot. Reload
    // first so a cached store that lags the persisted log (e.g. events appended
    // by another instance) still contains the matched event; otherwise
    // `snapshot_until` would silently produce an incomplete snapshot.
    let source_store = state
        .store_for_stream(source_stream_id, source_election, false)
        .await?;
    source_store.load().await?;

    let events = source_store.data.read().events.clone();
    let full_hash = events
        .iter()
        .find(|e| e.event_id == event_id)
        .map(|e| e.hash)
        .ok_or(AppError::GenericNotFound)?;
    let snapshot = PgStoreData::snapshot_until(&events, event_id);

    // Persist the import under a fresh CSB stream.
    let csb_store = CsbStore::acting_as(
        state
            .csb_store_for_stream(StreamId::new(), source_election)
            .await?,
        user,
    );
    csb_store
        .update(CsbAction::Import {
            hash: full_hash,
            source_stream_id,
            snapshot: Box::new(snapshot),
        })
        .await?;

    do_brp_verification(&csb_store, state.brp_client()).await?;

    Ok(ImportOutcome::Imported(redirect_success(
        CsbPoliticalGroupPath {
            stream_id: csb_store.stream_id,
        },
    )))
}

/// Create a new empty CSB store without importing from a political-group stream.
pub async fn create_empty<S: AppRequestState>(
    _: CsbCreateEmptyPath,
    State(state): State<S>,
    context: CsbContext,
) -> Result<Response, AppError> {
    let csb_store = CsbStore::acting_as(
        state
            .csb_store_for_stream(StreamId::new(), context.election)
            .await?,
        context.user()?,
    );
    csb_store.update(CsbAction::CreateEmpty).await?;
    Ok(redirect_success(CsbPoliticalGroupPath {
        stream_id: csb_store.stream_id,
    }))
}

/// Verifies every candidate against the BRP in a background task. Returns
/// immediately after spawning it instead of waiting for it to finish.
pub async fn do_brp_verification(store: &CsbStore, brp_client: &BrpClient) -> Result<(), AppError> {
    store
        .update(CsbAction::SetBrpStatus(BrpStatus::InProgress))
        .await?;

    // Spawned and intentionally not awaited here: verifying every candidate is
    // slow, and this must not block the request that triggered it.
    tokio::task::spawn(monitor_verification(store.clone(), brp_client.clone()));

    Ok(())
}

async fn monitor_verification(store: CsbStore, brp_client: BrpClient) {
    let outcome = tokio::task::spawn(verify_candidates(store.clone(), brp_client)).await;

    let error = match outcome {
        Ok(Ok(())) => return,
        Ok(Err(err)) => err.to_string(),
        Err(join_err) => join_err.to_string(),
    };

    if let Err(err) = store
        .update(CsbAction::SetBrpStatus(BrpStatus::Aborted(error)))
        .await
    {
        tracing::error!("failed to record aborted BRP status: {err}");
    }
}

/// Check every candidate on `store` not already covered by
/// `CsbStoreData::brp_validations` against the BRP, waiting
/// `BRP_COURTESY_TIMEOUT` between checks. A single candidate's failure is
/// logged and does not stop the rest of the sweep; only a failure to record
/// the final status is propagated to the caller.
async fn verify_candidates(store: CsbStore, brp_client: BrpClient) -> Result<(), AppError> {
    let already_validated = store.get_brp_validations();
    let unvalidated = store
        .get_persons(WithCorrections::All)
        .into_iter()
        .filter(|person| !already_validated.contains_key(&person.id));

    let mut ticker = tokio::time::interval(BRP_COURTESY_TIMEOUT);
    for person in unvalidated {
        ticker.tick().await;
        check_candidate(&store, &brp_client, &person).await;
    }

    tracing::info!(
        "Finished checking candidates on list {}",
        store
            .get_political_group(WithCorrections::All)
            .appellation
            .unwrap_or_default()
    );

    store
        .update(CsbAction::SetBrpStatus(BrpStatus::Finished))
        .await
}

/// Verify a single candidate, logging (rather than propagating) a failure so
/// it does not stop the rest of the sweep in [`verify_candidates`].
async fn check_candidate(store: &CsbStore, brp_client: &BrpClient, person: &Person) {
    match verify_candidate(store, brp_client, person).await {
        Err(err) => tracing::error!("{}", err),
        Ok(()) => tracing::info!("Checked person {} against the brp", person.id),
    }
}

/// Verify this candidate against the BRP, creating omissions that contain the
/// lists this candidate is on. Finally, the store is updated with a
/// `BrpPersonValidated` event.
async fn verify_candidate(
    store: &CsbStore,
    brp_client: &BrpClient,
    person: &Person,
) -> Result<(), AppError> {
    let candidate_lists = store
        .get_candidate_lists(WithCorrections::None)
        .iter()
        .filter(|cl| cl.candidates.contains(&person.id))
        .map(|cl| cl.id)
        .collect();

    let omissions = brp_client.verify(person, candidate_lists).await?;
    let valid = omissions.is_empty();

    for omission in omissions {
        omission.create(store).await?;
    }

    store
        .update(CsbAction::BrpPersonValidated {
            person: person.id,
            valid,
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::{
        AppState,
        CsbAction::Delete,
        CsbContext, ElectionConfig, PgEvent,
        test_utils::{response_body_string, sample_person_from_brp},
        utils::format_hash,
    };

    /// Populate a political-group stream with a single event in the (in-memory)
    /// test store and return its `(stream_id, formatted chain hash)`.
    async fn seed_source_event(state: &AppState) -> Result<(StreamId, String), AppError> {
        let source_stream = StreamId::new();
        let source_store = state
            .store_for_stream(source_stream, ElectionConfig::EK27, false)
            .await?;
        source_store.update(PgEvent::HideDownloadWarning).await?;

        let hash = source_store.data.read().events[0].hash;
        Ok((source_stream, format_hash(&hash, false)))
    }

    /// Poll briefly for the background BRP check to finish, instead of
    /// sleeping for the full courtesy timeout between candidates.
    async fn wait_for_brp_status_finished(store: &CsbStore) {
        for _ in 0..100 {
            if matches!(store.data.read().brp_validation_status, BrpStatus::Finished) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("BRP verification did not finish in time");
    }

    /// Submit the import form against a fresh test context.
    async fn submit(
        state: &AppState,
        hash: &str,
        confirmed_hash: Option<&str>,
    ) -> Result<Response, AppError> {
        Ok(import_submit(
            CsbImportPath {},
            State(state.clone()),
            CsbContext::new_test(),
            Form(ImportForm {
                hash: hash.to_string(),
                confirmed_hash: confirmed_hash.map(str::to_string),
            }),
        )
        .await?
        .into_response())
    }

    #[tokio::test]
    async fn import_renders_placeholder_page() -> Result<(), AppError> {
        let response = import(CsbImportPath {}, CsbContext::new_test())
            .await
            .unwrap()
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Import"));

        Ok(())
    }

    #[tokio::test]
    async fn import_submit_rejects_unparseable_hash() {
        let state = AppState::new_for_tests().await;
        let context = CsbContext::new_test();

        let response = import_submit(
            CsbImportPath {},
            State(state),
            context,
            Form(ImportForm {
                hash: "not-a-hash".to_string(),
                confirmed_hash: None,
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn import_submit_rejects_unknown_hash() {
        // A fresh in-memory backend has no cached political-group streams, so a
        // well-formed hash resolves to no package.
        let state = AppState::new_for_tests().await;
        let context = CsbContext::new_test();

        let response = import_submit(
            CsbImportPath {},
            State(state),
            context,
            Form(ImportForm {
                hash: "F381 3DE7 96D3 8033".to_string(),
                confirmed_hash: None,
            }),
        )
        .await
        .unwrap()
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn import_submit_imports_event_from_in_memory_store() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let (source_stream, hash) = seed_source_event(&state).await?;

        let context = CsbContext::new_test();

        let response = import_submit(
            CsbImportPath {},
            State(state.clone()),
            context,
            Form(ImportForm {
                hash,
                confirmed_hash: None,
            }),
        )
        .await?
        .into_response();

        // A successful import redirects to the examination overview.
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        // The import is recorded under a fresh CSB stream, carrying the source.
        let csb_stores = state.csb_store_registry().stores_by_scope().await?;
        assert_eq!(csb_stores.len(), 1);
        let imported = csb_stores[0].data.read().events.first().is_some_and(|e| {
            matches!(&e.payload.action, CsbAction::Import { source_stream_id, .. } if *source_stream_id == source_stream)
        });
        assert!(imported);

        Ok(())
    }

    #[tokio::test]
    async fn import_submit_warns_on_already_imported() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let (_, hash) = seed_source_event(&state).await?;

        // First import succeeds.
        let response = submit(&state, &hash, None).await?;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        // Re-importing the same source stream without confirmation re-renders
        // the form with a warning; nothing is imported yet.
        let response = submit(&state, &hash, None).await?;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("alert-warning"));
        assert!(body.contains(&hash));

        let csb_stores = state.csb_store_registry().stores_by_scope().await?;
        assert_eq!(csb_stores.len(), 1);

        // A confirmation for a different hash entry does not count.
        let response = submit(&state, &hash, Some("F381 3DE7 96D3 8033")).await?;
        assert_eq!(response.status(), StatusCode::OK);
        let csb_stores = state.csb_store_registry().stores_by_scope().await?;
        assert_eq!(csb_stores.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn import_submit_imports_already_imported_after_confirm() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let (_, hash) = seed_source_event(&state).await?;

        let response = submit(&state, &hash, None).await?;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        // Confirming the warned hash imports the same source stream again.
        let response = submit(&state, &hash, Some(&hash)).await?;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let csb_stores = state.csb_store_registry().stores_by_scope().await?;
        assert_eq!(csb_stores.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn create_empty_creates_csb_store_with_create_empty_event() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;

        let response = create_empty(
            CsbCreateEmptyPath {},
            State(state.clone()),
            CsbContext::new_test(),
        )
        .await?
        .into_response();

        // A successful creation redirects to the political group examination page.
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        // A single CSB store is recorded carrying the CreateEmpty event.
        let csb_stores = state.csb_store_registry().stores_by_scope().await?;
        assert_eq!(csb_stores.len(), 1);
        let data = csb_stores[0].data.read();
        assert!(matches!(
            data.events.first().unwrap().payload.action,
            CsbAction::CreateEmpty
        ));

        Ok(())
    }

    #[tokio::test]
    async fn brp_verification_records_an_omission_for_a_mismatched_person() -> Result<(), AppError>
    {
        let state = AppState::new_for_tests().await;
        let csb_store = state
            .csb_store_for_stream(StreamId::new(), ElectionConfig::EK27)
            .await?
            .acting_as_test_user();

        // This person matches a record in the local BRP mock exactly, except for
        // the tampered field below, so the check should find one mismatch.
        let mut person = sample_person_from_brp();
        person.address.house_number_addition = Some("nope".parse().unwrap());
        csb_store.add_person(person);

        let brp_client = BrpClient::new_for_test();
        do_brp_verification(&csb_store, &brp_client).await?;

        wait_for_brp_status_finished(&csb_store).await;
        let omission = csb_store.get_omission_for_test();
        assert_eq!(
            omission.description.as_str(),
            "De huisnummertoevoeging komt niet overeen met de BRP"
        );

        Ok(())
    }

    #[tokio::test]
    async fn do_brp_verification_returns_without_waiting_for_the_brp_check() -> Result<(), AppError>
    {
        let state = AppState::new_for_tests().await;
        let csb_store = state
            .csb_store_for_stream(StreamId::new(), ElectionConfig::EK27)
            .await?
            .acting_as_test_user();

        // Two candidates requires one BRP_COURTESY_TIMEOUT (1s) tick.
        // do_brp_verification should return well before that.
        csb_store.add_person(sample_person_from_brp());
        csb_store.add_person(sample_person_from_brp());

        let brp_client = BrpClient::new_for_test();

        let start = tokio::time::Instant::now();
        do_brp_verification(&csb_store, &brp_client).await?;
        let elapsed = start.elapsed();
        assert!(
            elapsed < BRP_COURTESY_TIMEOUT,
            "do_brp_verification should return immediately instead of waiting for the background check, took {elapsed:?}"
        );

        wait_for_brp_status_finished(&csb_store).await;

        Ok(())
    }

    #[tokio::test]
    async fn do_brp_verification_skips_already_validated_persons_on_a_later_call()
    -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let csb_store = state
            .csb_store_for_stream(StreamId::new(), ElectionConfig::EK27)
            .await?
            .acting_as_test_user();

        let mut person = sample_person_from_brp();
        person.address.house_number_addition = Some("nope".parse().unwrap());
        csb_store.add_person(person);

        let brp_client = BrpClient::new_for_test();

        do_brp_verification(&csb_store, &brp_client).await?;
        wait_for_brp_status_finished(&csb_store).await;
        assert_eq!(csb_store.data.read().omissions.len(), 1);
        assert_eq!(csb_store.data.read().brp_validations.len(), 1);

        // Re-running verification should skip the already-validated candidate
        // rather than re-checking them and recording a duplicate omission. With
        // no candidate left to check, the background task has nothing to wait
        // on, so a short fixed delay is enough instead of polling for
        // `Finished` (which the first run has already left behind).
        do_brp_verification(&csb_store, &brp_client).await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(csb_store.data.read().omissions.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn reimport_deleted_allowed() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let (_, hash) = seed_source_event(&state).await?;

        let context = CsbContext::new_test();
        let response = import_submit(
            CsbImportPath {},
            State(state.clone()),
            context,
            Form(ImportForm {
                hash: hash.clone(),
                confirmed_hash: None,
            }),
        )
        .await?
        .into_response();
        // First import succeeds (redirects away from import page)
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let csb_stores = state.csb_store_registry().stores_by_scope().await?;
        assert_eq!(csb_stores.len(), 1);

        // mark imported store as deleted
        csb_stores[0].update(Delete.by(CsbUser::new_test())).await?;

        // Re-importing the same source stream needs no confirmation when the
        // earlier import was deleted (the duplicate check consults the live
        // registry, as the in-memory backend persists nothing).
        let context = CsbContext::new_test();
        let response = import_submit(
            CsbImportPath {},
            State(state.clone()),
            context,
            Form(ImportForm {
                hash: hash.clone(),
                confirmed_hash: None,
            }),
        )
        .await?
        .into_response();

        // second import succeeds too as first import got deleted
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let csb_stores = state.csb_store_registry().stores_by_scope().await?;
        assert_eq!(csb_stores.iter().filter(|s| s.is_deleted()).count(), 1);
        assert_eq!(csb_stores.iter().filter(|s| !s.is_deleted()).count(), 1);
        for store in csb_stores {
            if let CsbAction::Import {
                hash: import_hash, ..
            } = store.data.read().events.first().unwrap().payload.action
            {
                assert_eq!(hash.clone(), format_hash(&import_hash, false));
            } else {
                panic!("No import")
            }
        }

        Ok(())
    }
}
