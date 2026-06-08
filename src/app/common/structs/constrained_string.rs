use std::str::FromStr;

use crate::{
    OptionAsStrExt,
    common::{InfoProblems, PotentialProblems, Problematic},
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

impl Problematic<()> for LegalName {
    fn get_problems(&self, _: ()) -> Vec<PotentialProblems> {
        if self.to_string().is_empty() {
            vec![PotentialProblems::NoLegalName]
        } else {
            Vec::new()
        }
    }
    
    fn get_info_problems(&self, _: ()) -> Vec<InfoProblems> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_char_is_valid() {
        assert_eq!(Ok(FirstName("A".to_string())), FirstName::from_str("A"));
        assert_eq!(Ok(LegalName("A".to_string())), LegalName::from_str("A"));
    }

    #[test]
    fn empty_is_rejected() {
        assert_eq!(
            Err(ValidationError::ValueShouldNotBeEmpty),
            LegalName::from_str("   ")
        );
    }

    #[test]
    fn too_long() {
        let long = "a".repeat(201);
        assert_eq!(
            Err(ValidationError::ValueTooLong(201, 200)),
            LegalName::from_str(&long)
        );
    }
}
