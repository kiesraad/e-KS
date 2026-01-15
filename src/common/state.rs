//! Application state container and request extractors.
//! Holds, among others: configuration, database pool, and CSRF tokens for handlers.

use std::sync::Arc;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::{PgPool, Postgres, pool::PoolConnection};

#[cfg(test)]
use crate::ElectionConfig;
use crate::{AppError, Config, CsrfTokens};

pub struct DbConnection(pub PoolConnection<Postgres>);

#[derive(FromRef, Clone)]
pub struct AppState {
    config: Arc<Config>,
    pool: sqlx::PgPool,
    csrf_tokens: CsrfTokens,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let config = Config::from_env()?;
        let pool = PgPool::connect_lazy(&config.database_url)?;
        let csrf_tokens = CsrfTokens::default();

        Ok(Self {
            config: Arc::new(config),
            pool,
            csrf_tokens,
        })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn csrf_tokens(&self) -> &CsrfTokens {
        &self.csrf_tokens
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    #[cfg(test)]
    pub fn new_for_tests(pool: PgPool) -> Self {
        let config = Config {
            database_url: "postgres://test".to_string(),
            election: ElectionConfig::EK2027,
        };

        Self {
            config: Arc::new(config),
            pool,
            csrf_tokens: CsrfTokens::default(),
        }
    }
}

impl<S> FromRequestParts<S> for DbConnection
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(_: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let conn = PgPool::from_ref(state).acquire().await?;

        Ok(DbConnection(conn))
    }
}

impl<S> FromRequestParts<S> for CsrfTokens
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(_: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        Ok(app_state.csrf_tokens.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn new_for_tests_sets_config_and_tokens(pool: PgPool) -> Result<(), sqlx::Error> {
        let state = AppState::new_for_tests(pool);

        assert_eq!(state.config().database_url, "postgres://test");
        assert!(matches!(state.config().election, ElectionConfig::EK2027));

        let token = state.csrf_tokens().issue();
        assert!(state.csrf_tokens().consume(&token.value));

        Ok(())
    }
}
