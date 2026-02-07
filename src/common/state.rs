//! Application state container and request extractors.
//! Holds, among others: configuration, database pool, and CSRF tokens for handlers.

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::PgPool;

use crate::{AppError, AppStore, Config, CsrfTokens};

#[derive(FromRef, Clone)]
pub struct AppState {
    pub store: AppStore,
    pub config: Config,
    pub csrf_tokens: CsrfTokens,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let config = Config::from_env()?;
        let pool = PgPool::connect_lazy(config.database_url)?;
        let csrf_tokens = CsrfTokens::default();
        let store = AppStore::new(pool.clone());

        Ok(Self {
            config,
            store,
            csrf_tokens,
        })
    }

    #[cfg(test)]
    pub async fn new_for_tests(pool: &PgPool) -> Self {
        use crate::AppEvent;

        let political_group_id = crate::political_groups::PoliticalGroupId::new();
        let political_group = crate::test_utils::sample_political_group(political_group_id);
        let store = AppStore::new(pool.clone());

        store
            .update(AppEvent::UpdatePoliticalGroup(political_group))
            .await
            .expect("store update");

        Self {
            config: Config::new_test(),
            store,
            csrf_tokens: CsrfTokens::default(),
        }
    }
}

impl<S> FromRequestParts<S> for Config
where
    S: Send + Sync,
    Config: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(_: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = Config::from_ref(state);

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn new_for_tests_sets_config_and_tokens(pool: PgPool) -> Result<(), sqlx::Error> {
        let state = AppState::new_for_tests(&pool).await;
        let config = Config::new_test();

        assert_eq!(state.config.database_url, config.database_url);

        let token = state.csrf_tokens.issue();
        assert!(state.csrf_tokens.consume(&token.value));

        Ok(())
    }

    #[sqlx::test]
    async fn config_from_request_parts_matches_state_config(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let state = AppState::new_for_tests(&pool).await;
        let (mut parts, _) = Request::new(()).into_parts();

        let config = Config::from_request_parts(&mut parts, &state)
            .await
            .expect("config");

        assert_eq!(config.database_url, state.config.database_url);

        Ok(())
    }
}
