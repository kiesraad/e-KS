use std::str::FromStr;

use crate::{
    OptionAsStrExt,
    common::{PotentialProblems, Problematic},
    form::{ValidationError, validate_length, validate_teletex_chars},
    transparent_string,
};

/// Define multiple separate types with the same basic constrains in the FromStr implementation
macro_rules! constrained_strings {
    ($(pub struct $name:ident;)*) => {
        $(
            transparent_string! {
                pub struct $name(String);
            }

            impl FromStr for $name {
                type Err = ValidationError;

                fn from_str(value: &str) -> Result<Self, Self::Err> {
                    let trimmed_value = validate_length(value, 1, 200)?;
                    validate_teletex_chars(&trimmed_value)?;
                    Ok($name(trimmed_value))
                }
            }
        )*
    };
}

constrained_strings! {
    pub struct FirstName;
    pub struct LegalName;
    pub struct StreetName;
    pub struct StateOrProvince;
}

impl Problematic for Option<LegalName> {
    fn get_problems(&self) -> Vec<PotentialProblems> {
        if self.is_empty_or_none() {
            vec![PotentialProblems::NoLegalName]
        } else {
            Vec::new()
        }
    }
}
