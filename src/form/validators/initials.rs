use crate::form::ValidationError;

/// Validates initials they should be uppercase alphanumeric and every initial should be followed by a point.
pub fn validate_initials() -> impl Fn(&str) -> Result<String, ValidationError> {
    |value: &str| {
        let initials = value.trim();

        if initials.is_empty() {
            return Err(ValidationError::ValueShouldNotBeEmpty);
        }

        let parts: Vec<&str> = initials
            .split('.')
            .filter(|part| !part.is_empty())
            .collect();

        for part in &parts {
            let chars: Vec<char> = part.chars().collect();
            if chars.len() != 1 || !chars[0].is_ascii_uppercase() {
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
    fn accepts_trimmed_uppercase_initials() {
        let result = (validate_initials())("  M.B.  ").unwrap();
        assert_eq!(result, "M.B.");
    }

    #[test]
    fn rejects_empty_initials() {
        let err = (validate_initials())("   ").unwrap_err();
        assert_eq!(err, ValidationError::ValueShouldNotBeEmpty);
    }

    #[test]
    fn rejects_lowercase_initials() {
        let err = (validate_initials())("M.b.").unwrap_err();
        assert_eq!(err, ValidationError::InvalidValue);
    }

    #[test]
    fn rejects_multi_character_segments() {
        let err = (validate_initials())("MB.").unwrap_err();
        assert_eq!(err, ValidationError::InvalidValue);
    }
}
