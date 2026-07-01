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

/// Events for the global CSB stream. Variants will be added as committee-wide
/// features are implemented (process steps, audit log, etc.).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CsbMainEvent {
    DeveloperLogin { stream_id: StreamId },
}
