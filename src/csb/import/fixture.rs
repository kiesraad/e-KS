use crate::{AppError, AppState, AppStoreData, CsbEvent, ElectionConfig, StreamId};

/// Marks CSB imports that were created from fixtures
pub const FIXTURE_IMPORT_HASH: [u8; 32] = [0; 32];

/// Create a political group stream with fixtures and import it as a CSB stream
/// if it doesn't exist already
pub async fn import_csb_fixture(
    state: &AppState,
    election: ElectionConfig,
) -> Result<(), AppError> {
    // Skip if a fixture import already exists in any committee-scoped stream.
    for store in state.csb_store_registry.stores_by_scope().await? {
        let comes_from_fixtures = store.data.read().events.first().is_some_and(
            |e| matches!(&e.payload, CsbEvent::Import { hash, .. } if *hash == FIXTURE_IMPORT_HASH),
        );
        if comes_from_fixtures {
            return Ok(());
        }
    }

    let pg_stream_id = StreamId::new();
    let app_store = state.store_for_stream(pg_stream_id, election, true).await?;
    let events = app_store.data.read().events.clone();
    let snapshot = AppStoreData::snapshot_until(&events, usize::MAX);

    state
        .csb_store_for_stream(StreamId::new(), election)
        .await?
        .update(CsbEvent::Import {
            hash: FIXTURE_IMPORT_HASH,
            source_stream_id: pg_stream_id,
            snapshot: Box::new(snapshot),
        })
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;

    #[tokio::test]
    async fn repeated_import_is_a_no_op() {
        let state = AppState::new_for_tests().await;
        let election = ElectionConfig::EK27;

        import_csb_fixture(&state, election).await.unwrap();
        import_csb_fixture(&state, election).await.unwrap();

        let csb_stores = state
            .csb_store_registry
            .stores_by_scope()
            .await
            .expect("csb stores");
        assert_eq!(
            csb_stores.len(),
            1,
            "a second fixture import should be skipped"
        );
    }
}
