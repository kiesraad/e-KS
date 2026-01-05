//! Locale detection and formatting helpers for request handling.
//! Extracted from Accept-Language headers and used by Context and templates.

use std::convert::Infallible;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub enum Locale {
    En,
    #[default]
    Nl,
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
        let locale = parts
            .headers
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok())
            .and_then(Locale::from_accept_language)
            .unwrap_or_default();

        Ok(locale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
