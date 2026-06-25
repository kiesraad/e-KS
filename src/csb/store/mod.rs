//! Event-sourced projection for the CSB (Centraal Stembureau) domain and the
//! request extractor that pulls a [`CsbStore`](crate::CsbStore) out of the
//! request extensions.

use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};

use crate::{
    AppError, AppStoreData, CsbEvent,
    store::{StoreData, StoreEvent},
};

/// Event-sourced domain projection for a single (stream, election) pair on the
/// CSB side.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CsbStoreData {
    pub(crate) imported_data: AppStoreData,
    pub(crate) events: Vec<StoreEvent<CsbEvent>>,
}

impl StoreData for CsbStoreData {
    type Event = CsbEvent;

    fn apply(&mut self, event: StoreEvent<CsbEvent>) {
        self.events.push(event.clone());

        match event.payload {
            CsbEvent::Import { snapshot, .. } => self.imported_data = *snapshot,
        }
    }

    fn events(&self) -> &[StoreEvent<Self::Event>] {
        &self.events
    }
}

// impl<S> FromRequestParts<S> for crate::CsbStore
// where
//     S: Send + Sync,
// {
//     type Rejection = AppError;

//     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
//         parts
//             .extensions
//             .get::<crate::CsbStore>()
//             .cloned()
//             .ok_or(AppError::Unauthorised)
//     }
// }

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
