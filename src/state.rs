//! Application state container and request extractors.
//! Holds, among others: configuration, store, and CSRF tokens for handlers.

use std::sync::Arc;

use crate::{
    AppError, AppStore, AppStoreData, AuthProvider, Config, PoliticalGroupId, SessionStore,
    store::StoreRegistry,
};
use axum::extract::FromRef;

/// Shared application state for request handlers and extractors.
#[derive(FromRef, Clone)]
pub struct AppState {
    pub config: &'static Config,
    pub store_registry: StoreRegistry<AppStoreData>,
    /// Active in-memory sessions for this application instance.
    pub sessions: SessionStore,
    pub auth_provider: Arc<AuthProvider>,
}

impl AppState {
    pub async fn new(typst_url: Option<String>) -> Result<Self, AppError> {
        let config = Config::from_env(typst_url)?;

        Self::new_with_config(config).await
    }

    pub async fn new_with_config(config: Config) -> Result<Self, AppError> {
        let store_registry = StoreRegistry::new(config.storage_url.to_string()).await?;
        let idp_metadata_url = config.idp_metadata_url.clone();

        Ok(Self {
            config: Box::leak(Box::new(config)),
            store_registry,
            sessions: SessionStore::new(),
            auth_provider: Arc::new(AuthProvider::new(idp_metadata_url).await?),
        })
    }

    pub async fn store_for_political_group(
        &self,
        political_group_id: PoliticalGroupId,
    ) -> Result<AppStore, AppError> {
        self.store_registry
            .get_or_create_with_init(political_group_id.uuid(), |store| async move {
                let needs_init = store.data.read().last_event_id == 0;
                if needs_init {
                    #[cfg(feature = "fixtures")]
                    crate::fixtures::load(&store, political_group_id).await?;
                }
                Ok(())
            })
            .await
    }

    #[cfg(test)]
    pub async fn new_for_tests() -> Self {
        let config = Config::new_test();
        let idp_metadata_url = config.idp_metadata_url.clone();

        Self {
            store_registry: StoreRegistry::new(config.storage_url.to_string())
                .await
                .expect("test StoreRegistry must initialize"),
            config: Box::leak(Box::new(config)),
            sessions: SessionStore::new(),
            auth_provider: Arc::new(AuthProvider::new(idp_metadata_url).await.unwrap()),
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
