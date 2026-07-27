//! Request extraction for the per-request [`PgStore`] and
//! shared boilerplate for `FromRequestParts` extractors that load a domain
//! object from it.

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{AppError, PgStore};

impl<S> FromRequestParts<S> for PgStore
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<PgStore>()
            .cloned()
            .ok_or(AppError::Unauthorised)
    }
}

/// Generate a [`FromRequestParts`] impl for
/// `$target`.
///
/// The body block is given the already-extracted request `store` plus the raw
/// `parts`/`state` (available inside `$body` for further `from_request_parts`
/// calls), and must evaluate to an [`AppResponse<Self>`](crate::AppResponse).
///
/// The four-binding form additionally extracts the request
/// [`Context`](crate::Context).
macro_rules! request_extractor {
    // Internal rule: emit the impl, splicing `$prelude` (the extraction
    // statements) ahead of the caller's body. Keeps the skeleton in one place.
    (@impl $target:ty, $parts:ident, $state:ident, { $($prelude:tt)* }, $body:block) => {
        impl<S: $crate::AppRequestState> axum::extract::FromRequestParts<S> for $target {
            type Rejection = $crate::AppError;

            async fn from_request_parts(
                $parts: &mut axum::http::request::Parts,
                $state: &S,
            ) -> Result<Self, Self::Rejection> {
                $($prelude)*
                $body
            }
        }
    };
    ($target:ty, |$store:ident, $parts:ident, $state:ident| $body:block) => {
        request_extractor!(@impl $target, $parts, $state, {
            let $store = <$crate::PgStore as axum::extract::FromRequestParts<S>>::from_request_parts(
                $parts, $state,
            )
            .await?;
        }, $body);
    };
    ($target:ty, |$store:ident, $context:ident, $parts:ident, $state:ident| $body:block) => {
        request_extractor!(@impl $target, $parts, $state, {
            let $store = <$crate::PgStore as axum::extract::FromRequestParts<S>>::from_request_parts(
                $parts, $state,
            )
            .await?;
            let $context = <$crate::Context as axum::extract::FromRequestParts<S>>::from_request_parts(
                $parts, $state,
            )
            .await?;
        }, $body);
    };
}

pub(crate) use request_extractor;
