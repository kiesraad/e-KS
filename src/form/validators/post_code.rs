use crate::form::ValidationError;

pub fn validate_post_code() -> impl Fn(&str) -> Result<String, ValidationError> {
    |value: &str| {
        let post_code = value.trim().replace(' ', "");

        if post_code.len() > 7 {
            return Err(ValidationError::ValueTooLong(post_code.len(), 7));
        }

        if post_code.len() < 6 {
            return Err(ValidationError::ValueTooShort(post_code.len(), 6));
        }

        let mut chars = post_code.chars().peekable();
        let mut numbers = (&mut chars).take(4);
        let numbers = if numbers.all(|c| c.is_ascii_digit()) {
            numbers.collect::<String>()
        } else {
            return Err(ValidationError::InvalidValue);
        };

        let mut letters = (&mut chars).take(2);
        let letters = if letters.all(|c| c.is_ascii_alphabetic()) {
            letters.collect::<String>()
        } else {
            return Err(ValidationError::InvalidValue);
        };

        Ok(format!("{} {}", numbers, letters.to_uppercase()))
    }
}

mod tests {
    use super::*;

    #[test]
    fn validate_post_code_validates_post_code() {
        assert_eq!(validate_post_code()("1234 AB"), Ok("1234 AB".to_string()));
    }
}
