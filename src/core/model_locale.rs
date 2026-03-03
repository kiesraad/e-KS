use serde::{Deserialize, Serialize};

use crate::Locale;

/// Locales for the web interface or the template
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnyLocale {
    En,
    Fry,
    Nl,
}

impl From<Locale> for AnyLocale {
    fn from(locale: Locale) -> Self {
        match locale {
            Locale::En => AnyLocale::En,
            Locale::Nl => AnyLocale::Nl,
        }
    }
}

/// Locales for the model templates
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelLocale {
    Fry,
    Nl,
}

impl std::fmt::Display for ModelLocale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelLocale::Fry => write!(f, "fry"),
            ModelLocale::Nl => write!(f, "nl"),
        }
    }
}

impl From<ModelLocale> for AnyLocale {
    fn from(locale: ModelLocale) -> Self {
        match locale {
            ModelLocale::Fry => AnyLocale::Fry,
            ModelLocale::Nl => AnyLocale::Nl,
        }
    }
}
