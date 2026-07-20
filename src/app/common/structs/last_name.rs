use std::str::FromStr;

use crate::{
    form::{ValidationError, validate_length, validate_teletex_chars},
    transparent_string,
};

use super::last_name_prefix::is_last_name_prefix;

transparent_string! {
    pub struct LastName(String);
}

impl FromStr for LastName {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = validate_length(value, 1, 255)?;
        validate_teletex_chars(&trimmed_value)?;

        if let Some((prefix, _)) = trimmed_value.split_once(' ')
            && is_last_name_prefix(prefix)
        {
            return Err(ValidationError::StartsWithLastNamePrefix);
        }

        Ok(LastName(trimmed_value))
    }
}
