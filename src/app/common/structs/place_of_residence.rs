//! Place of residence (the locality a person lives in).
//!
//! Unlike [`Locality`](super::Locality), a place of residence records whether
//! the (normalized) name was found in the BAG, so callers can flag values that
//! do not correspond to a known Dutch locality without rejecting them outright.
//!
//! Validation rules (via `FromStr`):
//! - Whitespace is trimmed; the value must be 2..=200 characters.
//! - Only Teletex characters are allowed.
//! - Known non-official names are replaced by their official counterpart
//!   (see [`replace_locality_alias`]).
//! - The normalized name is looked up in the BAG to pick the variant.
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::{
    form::{ValidationError, validate_length, validate_teletex_chars},
    utils::{bag, locality_aliases::replace_locality_alias},
};

/// A validated place of residence, tagged by whether it exists in the BAG.
///
/// Both variants wrap the same kind of normalized locality name; the
/// distinction only records the outcome of the BAG lookup performed while
/// parsing. Use [`Display`](std::fmt::Display), [`Deref`] or [`AsRef`] to read
/// the underlying name regardless of variant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum PlaceOfResidence {
    /// The name matches a locality known in the BAG.
    Known(String),
    /// A syntactically valid name with no matching BAG locality.
    Unknown(String),
}

impl std::fmt::Display for PlaceOfResidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaceOfResidence::Known(name) | PlaceOfResidence::Unknown(name) => write!(f, "{name}"),
        }
    }
}

impl Deref for PlaceOfResidence {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        match self {
            PlaceOfResidence::Known(name) | PlaceOfResidence::Unknown(name) => name,
        }
    }
}

impl AsRef<str> for PlaceOfResidence {
    fn as_ref(&self) -> &str {
        match self {
            PlaceOfResidence::Known(name) | PlaceOfResidence::Unknown(name) => name.as_str(),
        }
    }
}

impl std::str::FromStr for PlaceOfResidence {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = validate_length(value, 2, 200)?;
        validate_teletex_chars(&trimmed_value)?;

        let normalized = replace_locality_alias(&trimmed_value).unwrap_or(trimmed_value);

        if bag::locality_exists(&normalized, true, true) {
            Ok(PlaceOfResidence::Known(normalized))
        } else {
            Ok(PlaceOfResidence::Unknown(normalized))
        }
    }
}
