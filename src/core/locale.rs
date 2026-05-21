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

    pub(crate) fn from_language_code(code: &str) -> Option<Self> {
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
