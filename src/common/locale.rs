//! Locale detection and formatting helpers for request handling.
//! Extracted from Accept-Language headers and used by Context and templates.

use crate::{AppError, AppState};
use axum::{
    Router,
    extract::FromRequestParts,
    http::{header, request::Parts},
    response::Redirect,
};
use axum_extra::{
    TypedHeader,
    extract::{CookieJar, Form, cookie::Cookie},
    headers,
    routing::{RouterExt, TypedPath},
};
use serde::Deserialize;
use std::{convert::Infallible, str::FromStr};

static LOCALE_COOKIE_NAME: &str = "LANGUAGE";

#[derive(Default, Deserialize, Clone, Debug)]
struct LanguageSwitch {
    lang: Locale,
}

#[derive(TypedPath)]
#[typed_path("/language", rejection(AppError))]
pub struct SwitchLanguagePath;

async fn switch_language(
    _: SwitchLanguagePath,
    TypedHeader(referer): TypedHeader<headers::Referer>,
    mut cookie_jar: CookieJar,
    Form(form): Form<LanguageSwitch>,
) -> (CookieJar, Redirect) {
    cookie_jar = cookie_jar.add(Cookie::new(LOCALE_COOKIE_NAME, form.lang.as_str()));

    (cookie_jar, Redirect::to(&referer.to_string()))
}

#[derive(Default, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    En,
    #[default]
    Nl,
}

impl FromStr for Locale {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en" => Ok(Locale::En),
            "nl" => Ok(Locale::Nl),
            _ => Err("invalid locale"),
        }
    }
}

impl Locale {
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Nl => "nl",
        }
    }

    pub fn as_usize(&self) -> usize {
        match self {
            Locale::En => 0,
            Locale::Nl => 1,
        }
    }

    fn from_language_code(code: &str) -> Option<Self> {
        let code = code.to_ascii_lowercase();

        match code.as_str() {
            "en" => Some(Locale::En),
            "nl" => Some(Locale::Nl),
            _ if code.starts_with("en-") => Some(Locale::En),
            _ if code.starts_with("nl-") => Some(Locale::Nl),
            _ => None,
        }
    }

    fn from_accept_language(header_value: &str) -> Option<Self> {
        header_value
            .split(',')
            .find_map(|part| part.split(';').next())
            .and_then(|lang| Locale::from_language_code(lang.trim()))
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

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

pub fn locale_router() -> Router<AppState> {
    Router::new().typed_post(switch_language)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[test]
    fn converts_to_language_codes() {
        assert_eq!(Locale::En.as_str(), "en");
        assert_eq!(Locale::Nl.as_str(), "nl");
    }

    #[test]
    fn resolves_from_language_code_variants() {
        assert_eq!(Locale::from_language_code("EN"), Some(Locale::En));
        assert_eq!(Locale::from_language_code("nl-BE"), Some(Locale::Nl));
        assert_eq!(Locale::from_language_code("fr"), None);
    }

    #[test]
    fn resolves_from_accept_language_header() {
        let header = "nl-NL,nl;q=0.8,en;q=0.5";
        assert_eq!(Locale::from_accept_language(header), Some(Locale::Nl));

        let header = "fr-CA,fr;q=0.8,en;q=0.5";
        assert_eq!(Locale::from_accept_language(header), None);
    }

    #[tokio::test]
    async fn request_locale_prefers_cookie() {
        let request = Request::builder()
            .uri("/")
            .header(header::COOKIE, "LANGUAGE=en")
            .header(header::ACCEPT_LANGUAGE, "nl-NL,nl;q=0.8")
            .body(Body::empty())
            .unwrap();
        let (mut parts, _body) = request.into_parts();

        let locale = Locale::from_request_parts(&mut parts, &()).await.unwrap();

        assert_eq!(locale, Locale::En);
    }

    #[tokio::test]
    async fn request_locale_falls_back_to_accept_language() {
        let request = Request::builder()
            .uri("/")
            .header(header::COOKIE, "LANGUAGE=fr")
            .header(header::ACCEPT_LANGUAGE, "nl-NL,nl;q=0.8")
            .body(Body::empty())
            .unwrap();
        let (mut parts, _body) = request.into_parts();

        let locale = Locale::from_request_parts(&mut parts, &()).await.unwrap();

        assert_eq!(locale, Locale::Nl);
    }

    #[tokio::test]
    async fn switch_language_sets_cookie_and_redirects() {
        let app = locale_router().with_state(AppState::new_for_tests().await);

        let request = Request::builder()
            .method("POST")
            .uri("/language")
            .header(header::REFERER, "https://example.com/return")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from("lang=en"))
            .unwrap();

        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "https://example.com/return"
        );
        let set_cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert!(set_cookie.contains("LANGUAGE=en"));
    }
}
