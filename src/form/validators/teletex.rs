use crate::form::ValidationError;

/// Validate the characters are UTF8 but no different chars than those of the Teletex (T.61) standard are used
pub fn validate_teletex_chars() -> impl Fn(&str) -> Result<String, ValidationError> {
    |value: &str| {
        value.chars().try_for_each(|c| {
            let code = c as u32;
            if (32..127).contains(&code) || (161..383).contains(&code) {
                Ok(())
            } else {
                Err(ValidationError::InvalidValue)
            }
        })?;

        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ascii_printable() {
        let value = "Hello World! 123";
        let result = (validate_teletex_chars())(value).unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn accepts_extended_chars() {
        let value = "Caf\u{00E9} \u{00C5}";
        let result = (validate_teletex_chars())(value).unwrap();
        assert_eq!(result, value);
    }

    #[test]
    fn rejects_control_chars() {
        let err = (validate_teletex_chars())("Hi\nthere").unwrap_err();
        assert_eq!(err, ValidationError::InvalidValue);
    }

    #[test]
    fn rejects_chars_above_range() {
        let err = (validate_teletex_chars())("\u{0180}").unwrap_err();
        assert_eq!(err, ValidationError::InvalidValue);
    }
}
