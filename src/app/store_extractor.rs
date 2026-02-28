use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{AppError, AppStore};

impl<S> FromRequestParts<S> for AppStore
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AppStore>()
            .cloned()
            .ok_or(AppError::Unauthorised)
    }
}
