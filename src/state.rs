//! Application state container and request extractors.
//! Holds, among others: configuration, store, and CSRF tokens for handlers.

use crate::{
    AppError, AppStore, AppStoreData, BsnIdDeriver, Config, ElectionConfig, SessionStore, StreamId,
    common::Bsn,
    store::{EventEncryption, StoreRegistry},
};
use axum::extract::FromRef;

/// Shared application state for request handlers and extractors.
#[derive(FromRef, Clone)]
pub struct AppState {
    pub config: &'static Config,
    pub store_registry: StoreRegistry<AppStoreData>,
    /// Active in-memory sessions for this application instance.
    pub sessions: SessionStore,
    pub bsn_id_deriver: BsnIdDeriver,
}

impl AppState {
    pub async fn new(typst_url: Option<String>) -> Result<Self, AppError> {
        let config = Config::from_env(typst_url)?;

        Self::new_with_config(config).await
    }

    pub async fn new_with_config(config: Config) -> Result<Self, AppError> {
        let encryption = EventEncryption::new(&config.encryption_derivation_key);
        let store_registry = StoreRegistry::new(config.storage_url.to_string(), encryption).await?;
        let bsn_id_deriver = BsnIdDeriver::new(&config.id_derivation_key);

        Ok(Self {
            config: Box::leak(Box::new(config)),
            store_registry,
            sessions: SessionStore::new(),
            bsn_id_deriver,
        })
    }

    pub async fn store_for_stream(
        &self,
        stream_id: StreamId,
        election: ElectionConfig,
    ) -> Result<AppStore, AppError> {
        self.store_registry
            .get_or_create_with_init(stream_id.uuid(), election, |store| async move {
                let needs_init = store.data.read().events.is_empty();
                if needs_init {
                    store
                        .update(crate::AppEvent::StreamCreated { election })
                        .await?;
                    #[cfg(feature = "fixtures")]
                    crate::fixtures::load(&store).await?;
                }
                Ok(())
            })
            .await
    }

    /// Find which elections already have persisted data for the given BSN.
    pub async fn existing_elections_for_bsn(
        &self,
        bsn: &Bsn,
    ) -> Result<Vec<ElectionConfig>, AppError> {
        let all = ElectionConfig::all();
        let stream_ids: Vec<_> = all
            .iter()
            .map(|e| self.bsn_id_deriver.derive_stream_id(bsn, *e).uuid())
            .collect();

        let found = self.store_registry.streams_with_data(&stream_ids).await?;

        Ok(all
            .into_iter()
            .zip(stream_ids.iter())
            .filter(|(_, id)| found.contains(id))
            .map(|(election, _)| election)
            .collect())
    }

    #[cfg(test)]
    pub async fn new_for_tests() -> Self {
        let config = Config::new_test();
        let bsn_id_deriver = BsnIdDeriver::new(&config.id_derivation_key);
        let encryption = EventEncryption::new(&config.encryption_derivation_key);

        Self {
            store_registry: StoreRegistry::new(config.storage_url.to_string(), encryption)
                .await
                .expect("test StoreRegistry must initialize"),
            config: Box::leak(Box::new(config)),
            sessions: SessionStore::new(),
            bsn_id_deriver,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_for_tests_sets_config_and_tokens() -> Result<(), AppError> {
        let state = AppState::new_for_tests().await;
        let config = Config::new_test();

        assert_eq!(state.config.storage_url, config.storage_url);

        Ok(())
    }
}
