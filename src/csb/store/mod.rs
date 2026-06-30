//! Event-sourced projection for the CSB (Centraal Stembureau) domain and the
//! request extractor that pulls a [`CsbStore`](crate::CsbStore) out of the
//! request extensions.

use serde::{Deserialize, Serialize};

use crate::{
    AppStoreData, CsbEvent, Scope,
    store::{StoreData, StoreEvent},
};

/// Event-sourced domain projection for a single (stream, election) pair on the
/// CSB side.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CsbStoreData {
    pub(crate) imported_data: AppStoreData,
    pub(crate) events: Vec<StoreEvent<CsbEvent>>,
    pub(crate) is_examination_finished: bool,
}

impl StoreData for CsbStoreData {
    type Event = CsbEvent;

    fn apply(&mut self, event: StoreEvent<CsbEvent>) {
        self.events.push(event.clone());

        match event.payload {
            CsbEvent::Import { snapshot, .. } => self.imported_data = *snapshot,
            CsbEvent::ToggleFinish => self.is_examination_finished = !self.is_examination_finished,
        }
    }

    fn events(&self) -> &[StoreEvent<Self::Event>] {
        &self.events
    }

    fn scope() -> Scope {
        Scope::CentralElectoralCommittee
    }
}

#[cfg(test)]
impl crate::CsbStore {
    pub fn new_for_test() -> Self {
        use crate::StreamId;

        crate::store::Store {
            stream_id: StreamId::new(),
            election: crate::ElectionConfig::EK27,
            backend: crate::store::persistence::StoreBackend::Memory {
                store: crate::store::memory::MemoryStore::default(),
            },
            data: std::sync::Arc::new(parking_lot::RwLock::new(CsbStoreData::default())),
        }
    }
}
