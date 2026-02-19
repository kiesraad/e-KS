use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

use crate::{locale, trans};

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
    pub fn abbreviation(&self, locale: &locale::Locale) -> String {
        match self {
            Gender::Female => trans!("gender.f", locale),
            Gender::Male => trans!("gender.m", locale),
        }
    }
}
