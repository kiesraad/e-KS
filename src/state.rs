//! Application state container and request extractors.
//! Holds, among others: configuration, store, and CSRF tokens for handlers.

use crate::{
    AppError, AppStore, AppStoreData, Config, PoliticalGroupId, SessionStore,
    store::registry::StoreRegistry,
};
use axum::extract::FromRef;

/// Shared application state for request handlers and extractors.
#[derive(FromRef, Clone)]
pub struct AppState {
    pub config: Config,
    pub store_registry: StoreRegistry<AppStoreData>,
    /// Active in-memory sessions for this application instance.
    pub sessions: SessionStore,
}

impl AppState {
    pub async fn new_with_typst_url(typst_url: Option<String>) -> Result<Self, AppError> {
        let config = Config::from_env_with_typst_url(typst_url)?;
        let store_registry = StoreRegistry::new(config.storage_url.to_string());

        Ok(Self {
            config,
            store_registry,
            sessions: SessionStore::new(),
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
        Self {
            store_registry: StoreRegistry::new(config.storage_url.to_string()),
            config,
            sessions: SessionStore::new(),
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
