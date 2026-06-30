use crate::{AppError, AppState, AppStoreData, CsbEvent, ElectionConfig, StreamId};

/// Create a political group stream with fixtures and import it as a CSB stream.
pub async fn import_csb_fixture(
    state: &AppState,
    election: ElectionConfig,
) -> Result<(), AppError> {
    let pg_stream_id = StreamId::new();
    let app_store = state.store_for_stream(pg_stream_id, election, true).await?;
    let events = app_store.get_events();
    let snapshot = AppStoreData::snapshot_until(&events, usize::MAX);

    state
        .csb_store_for_stream(StreamId::new(), election)
        .await?
        .update(CsbEvent::Import {
            hash: "fixtures".to_string(),
            source_stream_id: pg_stream_id,
            snapshot: Box::new(snapshot),
        })
        .await?;

    Ok(())
}
