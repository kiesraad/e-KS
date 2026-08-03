use serde::Deserialize;

use crate::{common::DateOfBirth, persons::Person};

/// Which personal-data field of a candidate a correction dialog operates on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CandidateCorrectionField {
    Initials,
    LastName,
    DateOfBirth,
    PlaceOfResidence,
}

impl CandidateCorrectionField {
    pub fn label(self, locale: crate::Locale) -> String {
        match self {
            Self::Initials => crate::trans!("person.fields.initials", locale),
            Self::LastName => crate::trans!("person.fields.last_name", locale),
            Self::DateOfBirth => crate::trans!("person.fields.date_of_birth", locale),
            Self::PlaceOfResidence => crate::trans!("person.fields.place_of_residence", locale),
        }
    }

    /// Extract the string representation of this field from a person, using
    /// the same formatting as the examination pages and the correction overlay.
    pub fn extract(self, person: &Person) -> String {
        match self {
            Self::Initials => person.name.initials.to_string(),
            Self::LastName => person.name.last_name_with_prefix(),
            Self::DateOfBirth => DateOfBirth::format_option(&person.personal_data.date_of_birth),
            Self::PlaceOfResidence => person
                .personal_data
                .place_of_residence
                .as_ref()
                .map(|p| p.to_string())
                .unwrap_or_default(),
        }
    }
}

impl std::str::FromStr for CandidateCorrectionField {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "initials" => Ok(Self::Initials),
            "last-name" => Ok(Self::LastName),
            "date-of-birth" => Ok(Self::DateOfBirth),
            "place-of-residence" => Ok(Self::PlaceOfResidence),
            _ => Err("unknown correction field"),
        }
    }
}

impl std::fmt::Display for CandidateCorrectionField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initials => write!(f, "initials"),
            Self::LastName => write!(f, "last-name"),
            Self::DateOfBirth => write!(f, "date-of-birth"),
            Self::PlaceOfResidence => write!(f, "place-of-residence"),
        }
    }
}

impl<'de> Deserialize<'de> for CandidateCorrectionField {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}
