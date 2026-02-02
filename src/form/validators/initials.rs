use crate::form::{ValidationError, validators::teletex::is_teletex_char};

/// Validates initials they should be alphanumeric, including teletex chars and every initial should be followed by a point.
pub fn validate_initials() -> impl Fn(&str) -> Result<String, ValidationError> {
    |value: &str| {
        let initials = value.split_whitespace().collect::<String>();

        if initials.is_empty() {
            return Err(ValidationError::ValueShouldNotBeEmpty);
        }

        if initials.len() > 20 {
            return Err(ValidationError::ValueTooLong(20, initials.len()));
        }

        let parts: Vec<&str> = initials
            .split('.')
            .filter(|part| !part.is_empty())
            .collect();

        for part in &parts {
            let chars: Vec<char> = part.chars().collect();
            if chars.len() != 1 || !chars[0].is_alphanumeric() || !is_teletex_char(chars[0]) {
                return Err(ValidationError::InvalidValue);
            }
        }

        let result = parts.join(".") + ".";

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_trimmed_initials() {
        assert_eq!((validate_initials())("A.B").unwrap(), "A.B.");
        assert_eq!((validate_initials())("a.B.").unwrap(), "a.B.");
        assert_eq!((validate_initials())("  M.B.  ").unwrap(), "M.B.");
    }

    #[test]
    fn rejects_empty_initials() {
        let err = (validate_initials())("   ").unwrap_err();
        assert_eq!(err, ValidationError::ValueShouldNotBeEmpty);
    }

    #[test]
    fn rejects_special_characters() {
        let err = (validate_initials())("M.!.").unwrap_err();
        assert_eq!(err, ValidationError::InvalidValue);
    }

    #[test]
    fn rejects_multi_character_segments() {
        let err = (validate_initials())("MB.").unwrap_err();
        assert_eq!(err, ValidationError::InvalidValue);
    }
}
