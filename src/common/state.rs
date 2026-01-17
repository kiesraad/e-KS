//! Application state container and request extractors.
//! Holds, among others: configuration, database pool, and CSRF tokens for handlers.

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::{PgPool, Postgres, pool::PoolConnection};

use crate::{AppError, Config, CsrfTokens};

pub struct DbConnection(pub PoolConnection<Postgres>);

#[derive(FromRef, Clone)]
pub struct AppState {
    config: Config,
    pool: sqlx::PgPool,
    csrf_tokens: CsrfTokens,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let config = Config::from_env()?;
        let pool = PgPool::connect_lazy(config.database_url)?;
        let csrf_tokens = CsrfTokens::default();

        Ok(Self {
            config,
            pool,
            csrf_tokens,
        })
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub fn csrf_tokens(&self) -> CsrfTokens {
        self.csrf_tokens.clone()
    }

    pub fn config(&self) -> Config {
        self.config
    }

    #[cfg(test)]
    pub fn new_for_tests(pool: PgPool) -> Self {
        Self {
            config: Config::new_test(),
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

impl<S> FromRequestParts<S> for Config
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(_: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        Ok(app_state.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use sqlx::{Connection, PgPool};

    #[sqlx::test]
    async fn new_for_tests_sets_config_and_tokens(pool: PgPool) -> Result<(), sqlx::Error> {
        let state = AppState::new_for_tests(pool);
        let config = Config::new_test();

        assert_eq!(state.config().database_url, config.database_url);

        let token = state.csrf_tokens().issue();
        assert!(state.csrf_tokens().consume(&token.value));

        Ok(())
    }

    #[sqlx::test]
    async fn db_connection_from_request_parts_acquires_connection(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let state = AppState::new_for_tests(pool);
        let (mut parts, _) = Request::new(()).into_parts();

        let DbConnection(mut conn) = DbConnection::from_request_parts(&mut parts, &state)
            .await
            .expect("db connection");

        assert!(conn.ping().await.is_ok());

        Ok(())
    }

    #[sqlx::test]
    async fn csrf_tokens_from_request_parts_share_state_store(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let state = AppState::new_for_tests(pool);
        let (mut parts, _) = Request::new(()).into_parts();

        let tokens = CsrfTokens::from_request_parts(&mut parts, &state)
            .await
            .expect("csrf tokens");

        let token = tokens.issue();
        assert!(state.csrf_tokens().consume(&token.value));

        Ok(())
    }
}
