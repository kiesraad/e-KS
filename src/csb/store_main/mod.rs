mod event;
mod extractor;

pub use event::CsbMainEvent;

use serde::{Deserialize, Serialize};

use crate::{
    Scope, StreamId,
    store::{StoreData, StoreEvent},
};

/// Fixed stream ID shared by all CSB members for the global committee stream.
pub const CSB_MAIN_STREAM_ID: StreamId = StreamId(uuid::Uuid::from_u128(
    0xC5B0_0000_0000_8000_8000_0000_0000_0001,
));

/// Global CSB state shared across all committee members: process step tracking,
/// audit log entries (logins, imports, etc.), and other committee-wide events.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CsbMainStoreData {
    pub(crate) events: Vec<StoreEvent<CsbMainEvent>>,
}

impl StoreData for CsbMainStoreData {
    type Event = CsbMainEvent;

    fn apply(&mut self, event: StoreEvent<CsbMainEvent>) {
        self.events.push(event.clone());
        match event.payload {
            CsbMainEvent::DeveloperLogin { .. } => {}
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
impl crate::CsbMainStore {
    pub fn new_for_test() -> Self {
        use crate::ElectionConfig;

        crate::store::Store {
            stream_id: CSB_MAIN_STREAM_ID,
            election: ElectionConfig::EK27,
            backend: crate::store::persistence::StoreBackend::Memory {
                store: crate::store::memory::MemoryStore::default(),
            },
            data: std::sync::Arc::new(parking_lot::RwLock::new(CsbMainStoreData::default())),
        }
    }
}
