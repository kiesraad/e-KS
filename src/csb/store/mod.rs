//! Event-sourced projection for the CSB (Centraal Stembureau) domain and the
//! request extractor that pulls a [`CsbStore`](crate::CsbStore) out of the
//! request extensions.

use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};

use crate::{
    AppError, CsbEvent,
    store::{StoreData, StoreEvent},
};

/// Event-sourced domain projection for a single (stream, election) pair on the
/// CSB side.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CsbStoreData {
    /// Chain hashes of the packages that have been imported, in order.
    pub(crate) imported_hashes: Vec<String>,
    pub(crate) events: Vec<StoreEvent<CsbEvent>>,
}

impl StoreData for CsbStoreData {
    type Event = CsbEvent;

    fn apply(&mut self, event: StoreEvent<CsbEvent>) {
        self.events.push(event.clone());

        match event.payload {
            CsbEvent::Import { hash, .. } => self.imported_hashes.push(hash),
        }
    }

    fn events(&self) -> &[StoreEvent<Self::Event>] {
        &self.events
    }
}

impl<S> FromRequestParts<S> for crate::CsbStore
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<crate::CsbStore>()
            .cloned()
            .ok_or(AppError::Unauthorised)
    }
}

#[cfg(test)]
impl crate::CsbStore {
    pub fn new_for_test() -> Self {
        crate::store::Store {
            stream_id: uuid::Uuid::new_v4(),
            election: crate::ElectionConfig::EK27,
            backend: crate::store::persistence::StoreBackend::Memory,
            data: std::sync::Arc::new(parking_lot::RwLock::new(CsbStoreData::default())),
        }
    }
}
