use crate::form::ValidationError;

pub fn validate_length(
    value: &str,
    min_length: usize,
    max_length: usize,
) -> Result<String, ValidationError> {
    let trimmed_value = value.trim();

    if trimmed_value.is_empty() {
        return Err(ValidationError::ValueShouldNotBeEmpty);
    }

    if trimmed_value.len() < min_length {
        return Err(ValidationError::ValueTooShort(
            trimmed_value.len(),
            min_length,
        ));
    }

    if trimmed_value.len() > max_length {
        return Err(ValidationError::ValueTooLong(
            trimmed_value.len(),
            max_length,
        ));
    }

    Ok(trimmed_value.to_string())
}

pub fn validate_teletex_chars(value: &str) -> Result<(), ValidationError> {
    value.chars().try_for_each(|c| {
        if is_teletex_char(c) {
            Ok(())
        } else {
            Err(ValidationError::InvalidValue)
        }
    })
}

pub fn is_teletex_char(c: char) -> bool {
    let code = c as u32;
    (32..127).contains(&code) || (161..383).contains(&code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::form::ValidationError;

    #[test]
    fn validate_length_trims_and_accepts_in_range() {
        let value = "  hello  ";
        let result = validate_length(value, 3, 10).expect("valid length");
        assert_eq!(result, "hello");
    }

    #[test]
    fn validate_length_rejects_empty_after_trim() {
        let err = validate_length("   ", 1, 5).expect_err("empty");
        assert_eq!(err, ValidationError::ValueShouldNotBeEmpty);
    }

    #[test]
    fn validate_length_rejects_too_short_and_too_long() {
        let err = validate_length("ab", 3, 5).expect_err("too short");
        assert_eq!(err, ValidationError::ValueTooShort(2, 3));

        let err = validate_length("abcdef", 1, 5).expect_err("too long");
        assert_eq!(err, ValidationError::ValueTooLong(6, 5));
    }

    #[test]
    fn validate_teletex_chars_accepts_valid_ranges() {
        let value = "Az 0~\u{00A1}\u{00FE}";
        assert!(validate_teletex_chars(value).is_ok());
    }

    #[test]
    fn validate_teletex_chars_rejects_invalid_chars() {
        let err = validate_teletex_chars("\u{001F}").expect_err("control char");
        assert_eq!(err, ValidationError::InvalidValue);

        let err = validate_teletex_chars("\u{00A0}").expect_err("gap char");
        assert_eq!(err, ValidationError::InvalidValue);

        let err = validate_teletex_chars("\u{017F}").expect_err("upper bound");
        assert_eq!(err, ValidationError::InvalidValue);
    }

    #[test]
    fn is_teletex_char_respects_boundaries() {
        assert!(is_teletex_char(' '));
        assert!(is_teletex_char('~'));
        assert!(!is_teletex_char('\u{007F}'));
        assert!(!is_teletex_char('\u{00A0}'));
        assert!(is_teletex_char('\u{00A1}'));
        assert!(is_teletex_char('\u{017E}'));
        assert!(!is_teletex_char('\u{017F}'));
    }
}
