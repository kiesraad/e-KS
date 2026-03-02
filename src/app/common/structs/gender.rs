use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

use crate::core::AnyLocale;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, FromStr,
)]
#[serde(rename_all = "lowercase")]
#[display(rename_all = "lowercase")]
#[from_str(rename_all = "lowercase")]
pub enum Gender {
    Female,
    Male,
}

impl Gender {
    pub fn abbreviation(&self, locale: AnyLocale) -> &'static str {
        match self {
            Gender::Female => match locale {
                AnyLocale::En | AnyLocale::Fry => "f",
                AnyLocale::Nl => "v",
            },
            Gender::Male => match locale {
                AnyLocale::En | AnyLocale::Fry | AnyLocale::Nl => "m",
            },
        }
    }
}
