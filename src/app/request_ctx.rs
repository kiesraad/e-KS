//! Bundled handler extractor: template [`Context`], event [`AppStore`], and
//! [`QueryParamState`]. Use in place of extracting all three separately to keep
//! handler signatures tight — destructure at the top of the handler when you
//! need the inner fields.

use axum::{
    extract::{FromRequestParts, Query},
    http::request::Parts,
};

use crate::{
    AppError, AppStore, Context, CsrfTokens, ElectionConfig, Locale, QueryParamState,
    form::{FormData, WithCsrfToken},
};

pub struct RequestCtx {
    pub context: Context,
    pub store: AppStore,
    pub query: QueryParamState,
}

impl RequestCtx {
    pub fn csrf(&self) -> &CsrfTokens {
        &self.context.session.csrf_tokens
    }

    pub fn locale(&self) -> Locale {
        self.context.session.locale
    }

    pub fn election(&self) -> &ElectionConfig {
        &self.context.election
    }

    pub fn form_data<T: Default + WithCsrfToken>(&self) -> FormData<T> {
        FormData::new(self.csrf())
    }

    pub fn form_data_with<T: WithCsrfToken>(&self, data: T) -> FormData<T> {
        FormData::new_with_data(data, self.csrf())
    }
}

impl<S> FromRequestParts<S> for RequestCtx
where
    S: Clone + Send + Sync + 'static,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let context = Context::from_request_parts(parts, state).await?;
        let store = AppStore::from_request_parts(parts, state).await?;
        let Query(query) = Query::<QueryParamState>::from_request_parts(parts, state).await?;
        Ok(Self {
            context,
            store,
            query,
        })
    }
}
