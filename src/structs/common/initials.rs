//! Initials in normalized form (e.g. `A.B.`, `S.Q`, `J P`).
//!
//! The rules follow the way the BRP derives initials from first names, see
//! <https://developer.rvig.nl/brp-api/personen/features/voorletters/>:
//! - One initial per first name, so a first name starting with a diphthong
//!   (`Th`, `Ph`, `Ch`, `IJ`, etc.) or a compound first name is abbreviated to
//!   a single initial as well.
//! - An initial is followed by a dot, unless the first name consists of a
//!   single letter (`Suzie Q` becomes `S.Q`).
//! - An initial that is not followed by a dot is separated from the next
//!   initial by a space (`J P`).
//!
//! Accepted format rules (see `FromStr`):
//! - One or more initials, each exactly one alphanumeric teletex character
//!   (which includes letters with diacritics), optionally followed by a dot.
//! - Initials that are not followed by a dot are separated by whitespace,
//!   which is normalized to a single space.
//! - Superfluous whitespace is removed.
//! - At most 20 initials, punctuation and whitespace do not count.
use crate::{
    form::{ValidationError, is_teletex_char},
    transparent_string,
};

/// The BRP allows 40 characters, which fits 20 initials followed by a dot
const MAX_INITIALS: usize = 20;

transparent_string! {
    pub struct Initials(String);
}

impl std::str::FromStr for Initials {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // Every initial is a single character, with a flag telling whether it is followed by a dot
        let mut initials: Vec<(char, bool)> = Vec::new();
        let mut chars = value.chars().peekable();
        // Whether the next initial is preceded by a separator (a dot or whitespace)
        let mut separated = true;

        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                separated = true;
                continue;
            }

            if !separated || !is_initial_char(c) {
                return Err(ValidationError::InvalidValue);
            }

            // A dot both terminates the initial and separates it from the next one
            separated = chars.next_if_eq(&'.').is_some();
            initials.push((c, separated));
        }

        if initials.is_empty() {
            return Err(ValidationError::ValueShouldNotBeEmpty);
        }

        if initials.len() > MAX_INITIALS {
            return Err(ValidationError::TooManyInitials(
                initials.len(),
                MAX_INITIALS,
            ));
        }

        let mut result = String::new();
        let mut previous_has_dot = true;

        for (initial, has_dot) in initials {
            if !previous_has_dot {
                result.push(' ');
            }

            result.push(initial);

            if has_dot {
                result.push('.');
            }

            previous_has_dot = has_dot;
        }

        Ok(Initials(result))
    }
}

fn is_initial_char(c: char) -> bool {
    c.is_alphanumeric() && is_teletex_char(c)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn parse(value: &str) -> Result<String, ValidationError> {
        Initials::from_str(value).map(|initials| initials.to_string())
    }

    #[test]
    fn accepts_dotted_initials() {
        assert_eq!(parse("H."), Ok("H.".to_string()));
        assert_eq!(parse("A.C."), Ok("A.C.".to_string()));
        assert_eq!(parse("J.F.R."), Ok("J.F.R.".to_string()));
    }

    #[test]
    fn accepts_initials_of_single_letter_first_names() {
        // `Suzie Q`, `A` and `J P` respectively
        assert_eq!(parse("S.Q"), Ok("S.Q".to_string()));
        assert_eq!(parse("A"), Ok("A".to_string()));
        assert_eq!(parse("J P"), Ok("J P".to_string()));
    }

    #[test]
    fn accepts_letters_with_diacritics() {
        assert_eq!(parse("É.Ø."), Ok("É.Ø.".to_string()));
        assert_eq!(parse("ž"), Ok("ž".to_string()));
    }

    #[test]
    fn normalizes_superfluous_whitespace() {
        assert_eq!(parse("  A. B.  "), Ok("A.B.".to_string()));
        assert_eq!(parse("A.  B"), Ok("A.B".to_string()));
        assert_eq!(parse("J\tP"), Ok("J P".to_string()));
        assert_eq!(parse("A  B."), Ok("A B.".to_string()));
    }

    #[test]
    fn rejects_empty_values() {
        assert_eq!(parse(""), Err(ValidationError::ValueShouldNotBeEmpty));
        assert_eq!(parse("  "), Err(ValidationError::ValueShouldNotBeEmpty));
    }

    #[test]
    fn rejects_initials_of_more_than_one_character() {
        // Diphthongs are abbreviated to a single initial
        assert_eq!(parse("IJ."), Err(ValidationError::InvalidValue));
        assert_eq!(parse("Th.P."), Err(ValidationError::InvalidValue));
        // Without a separator it is unclear where an initial ends
        assert_eq!(parse("AB"), Err(ValidationError::InvalidValue));
    }

    #[test]
    fn rejects_other_punctuation() {
        assert_eq!(parse("A..B."), Err(ValidationError::InvalidValue));
        assert_eq!(parse("A .B."), Err(ValidationError::InvalidValue));
        assert_eq!(parse(".A."), Err(ValidationError::InvalidValue));
        assert_eq!(parse("A-B."), Err(ValidationError::InvalidValue));
        assert_eq!(parse("A,B."), Err(ValidationError::InvalidValue));
    }

    #[test]
    fn rejects_non_teletex_characters() {
        assert_eq!(parse("Ω."), Err(ValidationError::InvalidValue));
        assert_eq!(parse("\u{017F}."), Err(ValidationError::InvalidValue));
    }

    #[test]
    fn counts_initials_instead_of_characters_for_the_maximum() {
        let twenty = "A.".repeat(20);
        assert_eq!(parse(&twenty), Ok(twenty.clone()));

        assert_eq!(
            parse(&"A.".repeat(21)),
            Err(ValidationError::TooManyInitials(21, 20))
        );
    }
}
