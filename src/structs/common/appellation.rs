use std::str::FromStr;

use crate::{
    OptionAsStrExt,
    form::{ValidationError, validate_teletex_chars},
    structs::{
        common::{PotentialProblems, Problematic, Problems},
        list_designation::ListDesignation,
    },
    transparent_string,
};

transparent_string! {
    pub struct Appellation(String);
}

impl Appellation {
    /// The maximum number of character a appellation can consist of (excluding spaces)
    pub const MAX_CHAR_COUNT: usize = 35;
}

impl FromStr for Appellation {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let words: Vec<_> = value.split_whitespace().collect();
        let trimmed_value = words.join(" ");
        let char_count: usize = words.iter().map(|w| w.chars().count()).sum();

        if char_count < 1 {
            return Err(ValidationError::ValueTooShort(char_count, 1));
        }

        if char_count > Self::MAX_CHAR_COUNT {
            return Err(ValidationError::ValueTooLong(
                char_count,
                Self::MAX_CHAR_COUNT,
            ));
        }
        validate_teletex_chars(&trimmed_value)?;
        Ok(Appellation(trimmed_value))
    }
}

impl Problematic<Option<ListDesignation>> for Option<Appellation> {
    fn get_problems(&self, list_designation: Option<ListDesignation>) -> Problems {
        let potential_problems =
            if list_designation != Some(ListDesignation::Blank) && self.is_empty_or_none() {
                vec![PotentialProblems::NoAppellation]
            } else {
                Vec::new()
            };
        Problems {
            potential_problems,
            info_problems: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_name() {
        assert_eq!(
            Ok(Appellation("De Tegen Partij".to_string())),
            Appellation::from_str("De Tegen Partij")
        );
    }

    #[test]
    fn valid_name_with_extra_spaces() {
        assert_eq!(
            Ok(Appellation("De Tegen Partij".to_string())),
            Appellation::from_str("\t  De  \t  Tegen   Partij ")
        );

        assert_eq!(
            Ok(Appellation("De Tegen Partij".to_string())),
            Appellation::from_str("\t  De  \t  Tegen   Partij \t")
        );
    }

    #[test]
    fn too_long() {
        assert_eq!(
            Err(ValidationError::ValueTooLong(36, 35)),
            Appellation::from_str("a string of exactly 36 chars long ex. spaces")
        );
    }

    #[test]
    fn too_short() {
        assert_eq!(
            Err(ValidationError::ValueTooShort(0, 1)),
            Appellation::from_str("        ")
        );
    }

    #[test]
    fn single_char_is_valid() {
        assert_eq!(Ok(Appellation("A".to_string())), Appellation::from_str("A"));
    }
}
