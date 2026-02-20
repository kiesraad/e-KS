use crate::{Locale, common::LOCALE_COOKIE_NAME};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use axum_extra::extract::CookieJar;
use std::convert::Infallible;

impl<S> FromRequestParts<S> for Locale
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let cookies = CookieJar::from_request_parts(parts, _state).await?;
        let cookie: Option<Locale> = cookies
            .get(LOCALE_COOKIE_NAME)
            .and_then(|cookie| cookie.value().parse().ok());

        if let Some(locale) = cookie {
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
