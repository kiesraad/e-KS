//! [`PgStore`]: the store handle used by the app feature handlers, including
//! the CSB paper-corrections write target.

use crate::{AppError, CsbAction, CsbStore, MAX_CANDIDATES, PgEvent, PgStoreData, store::Store};

/// Store handle used by the app feature handlers: reads come from the
/// [`PgStoreData`] projection, writes dispatch [`PgEvent`]s to the write
/// target.
///
/// A political group session appends events to its own stream. A CSB session
/// correcting the paper documents gets every event wrapped in
/// [`CsbAction::PaperCorrectedUpdate`] and appended to the CSB stream instead,
/// so corrections land in that stream's `paper_corrected_data` projection and
/// the political group's own stream is never touched.
#[derive(Clone)]
pub struct PgStore {
    /// Projection served to reads; [`std::ops::Deref`] keeps `store.election`,
    /// `store.data` and the getters working unchanged.
    projection: Store<PgStoreData>,
    target: WriteTarget,
}

#[derive(Clone)]
enum WriteTarget {
    /// Persist events on the projection's own stream.
    Own,
    /// Wrap events in [`CsbAction::PaperCorrectedUpdate`] and persist them on
    /// this CSB stream, which records the correcting committee member. The
    /// projection is a request-local snapshot of the stream's
    /// `paper_corrected_data`.
    PaperCorrections { store: CsbStore },
}

impl std::ops::Deref for PgStore {
    type Target = Store<PgStoreData>;

    fn deref(&self) -> &Self::Target {
        &self.projection
    }
}

impl PgStore {
    /// Wrap a store that persists app events on its own stream.
    pub fn own(store: Store<PgStoreData>) -> Self {
        Self {
            projection: store,
            target: WriteTarget::Own,
        }
    }

    /// Build a paper-corrections handle over a loaded CSB store: reads serve a
    /// snapshot of its `paper_corrected_data`, writes go to the CSB stream,
    /// recorded as triggered by the store's committee member.
    pub fn paper_corrections(csb_store: CsbStore) -> Self {
        let projection = Store::new_for_temp_stream(csb_store.election);
        *projection.data.write() = csb_store.data.read().paper_corrected_data.clone();

        Self {
            projection: Store {
                stream_id: csb_store.stream_id,
                ..projection
            },
            target: WriteTarget::PaperCorrections { store: csb_store },
        }
    }

    /// Create a temporary, in-memory handle with no persistence.
    pub fn new_for_temp_stream(election: crate::ElectionConfig) -> Self {
        Self::own(Store::new_for_temp_stream(election))
    }

    /// Persist an event on the write target and apply it to the projection.
    pub async fn update(&self, event: PgEvent) -> Result<(), AppError> {
        match &self.target {
            WriteTarget::Own => self.projection.update(event).await,
            WriteTarget::PaperCorrections { store: csb_store } => {
                csb_store
                    .update(CsbAction::PaperCorrectedUpdate(Box::new(event)))
                    .await?;

                // Refresh the snapshot so reads later in this request observe
                // the correction.
                *self.projection.data.write() = csb_store.data.read().paper_corrected_data.clone();

                Ok(())
            }
        }
    }

    /// Hard cap on the number of candidates a single list may hold.
    ///
    /// Paper corrections record the list exactly as it was handed in on paper,
    /// so an over-long list has to be enterable there: the surplus candidates
    /// are scratched later, they are not kept out of the data. The cap only
    /// applies to a political group entering its own data.
    pub fn candidate_limit(&self) -> usize {
        match &self.target {
            WriteTarget::Own => MAX_CANDIDATES,
            WriteTarget::PaperCorrections { .. } => usize::MAX,
        }
    }

    /// The CSB stream receiving paper corrections, when this handle is in
    /// paper-corrections mode.
    pub fn paper_corrections_stream_id(&self) -> Option<crate::StreamId> {
        match &self.target {
            WriteTarget::Own => None,
            WriteTarget::PaperCorrections {
                store: csb_store, ..
            } => Some(csb_store.stream_id),
        }
    }

    /// The imported snapshot the paper corrections were applied on top of,
    /// when this handle is in paper-corrections mode. Serves as the base
    /// state for audit-log replays, since the correction events alone do not
    /// reconstruct the imported entities.
    pub fn imported_snapshot(&self) -> Option<PgStoreData> {
        match &self.target {
            WriteTarget::Own => None,
            WriteTarget::PaperCorrections {
                store: csb_store, ..
            } => Some(csb_store.data.read().imported_data.clone()),
        }
    }
}

#[cfg(all(test, feature = "database"))]
impl PgStore {
    pub async fn new_with_pool_for_stream(
        pool: sqlx::PgPool,
        stream_id: crate::StreamId,
        election: crate::ElectionConfig,
        master: &crate::crypto::MasterKey,
    ) -> Result<Self, AppError> {
        Ok(Self::own(
            Store::new_with_pool_for_stream(pool, stream_id, election, master).await?,
        ))
    }
}

#[cfg(test)]
impl PgStore {
    pub fn new_for_test() -> Self {
        Self::new_for_test_with_election(crate::ElectionConfig::EK27)
    }

    pub fn new_for_test_with_election(election: crate::ElectionConfig) -> Self {
        use crate::StreamId;

        let data = PgStoreData {
            political_group: crate::test_utils::sample_political_group(),
            ..PgStoreData::default()
        };

        Self::own(crate::store::Store {
            stream_id: StreamId::new(),
            election,
            backend: crate::store::StoreBackend::Memory {
                store: crate::store::memory::MemoryStore::default(),
            },
            data: std::sync::Arc::new(parking_lot::RwLock::new(data)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hard maximum guards a political group's own data; paper
    /// corrections must be able to record an over-long list as handed in.
    #[tokio::test]
    async fn candidate_limit_is_lifted_while_correcting() -> Result<(), AppError> {
        assert_eq!(PgStore::new_for_test().candidate_limit(), MAX_CANDIDATES);
        assert_eq!(
            crate::test_utils::paper_corrections_store()
                .await?
                .candidate_limit(),
            usize::MAX
        );

        Ok(())
    }
}
