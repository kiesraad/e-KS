use crate::{Locale, Session};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use std::convert::Infallible;

impl<S> FromRequestParts<S> for Locale
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(locale) = parts
            .extensions
            .get::<Session>()
            .map(|session| session.locale)
        {
            return Ok(locale);
        }

        let locale = parts
            .headers
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok())
            .and_then(Locale::from_accept_language)
            .unwrap_or_default();

        Ok(locale)
    }
}
