//! Locale detection and formatting helpers for request handling.
//! Extracted from Accept-Language headers and used by Context and templates.

use serde::Deserialize;
use std::str::FromStr;

/// Supported UI locales for requests and templates.
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

    pub fn from_accept_language(header_value: &str) -> Option<Self> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;
    use axum::{
        body::Body,
        extract::FromRequestParts,
        http::{Request, header},
    };

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
    async fn request_locale_prefers_session() {
        let mut request = Request::builder()
            .uri("/")
            .header(header::ACCEPT_LANGUAGE, "nl-NL,nl;q=0.8")
            .body(Body::empty())
            .unwrap();
        request
            .extensions_mut()
            .insert(Session::new_with_locale(Locale::En));
        let (mut parts, _body) = request.into_parts();

        let locale = Locale::from_request_parts(&mut parts, &()).await.unwrap();

        assert_eq!(locale, Locale::En);
    }

    #[tokio::test]
    async fn request_locale_falls_back_to_accept_language() {
        let request = Request::builder()
            .uri("/")
            .header(header::ACCEPT_LANGUAGE, "nl-NL,nl;q=0.8")
            .body(Body::empty())
            .unwrap();
        let (mut parts, _body) = request.into_parts();

        let locale = Locale::from_request_parts(&mut parts, &()).await.unwrap();

        assert_eq!(locale, Locale::Nl);
    }
}
