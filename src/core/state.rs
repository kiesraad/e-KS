//! Application state container and request extractors.
//! Holds, among others: configuration, store, and CSRF tokens for handlers.

use crate::{AppError, Config, CsrfTokens, Store};
use axum::extract::FromRef;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub store: Store,
    pub config: Config,
    pub csrf_tokens: CsrfTokens,
}

impl AppState {
    pub async fn new() -> Result<Self, AppError> {
        let config = Config::from_env()?;
        let csrf_tokens = CsrfTokens::default();
        let store = Store::new(config.storage_url).await?;

        Ok(Self {
            config,
            store,
            csrf_tokens,
        })
    }

    #[cfg(test)]
    pub async fn new_for_tests() -> Self {
        Self {
            config: Config::new_test(),
            store: Store::new_for_test().await,
            csrf_tokens: CsrfTokens::default(),
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

        let token = state.csrf_tokens.issue();
        assert!(state.csrf_tokens.consume(&token.value));

        Ok(())
    }
}
