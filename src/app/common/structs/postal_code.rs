use derive_more::{Deref, Display, Into};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::form::ValidationError;

#[derive(Default, Debug, Deref, Clone, Into, Display, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PostalCode(String);

impl FromStr for PostalCode {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let postal_code = value.split_whitespace().collect::<String>();

        if postal_code.len() != 6 {
            return Err(ValidationError::InvalidPostalCode);
        }

        let bytes = postal_code.as_bytes();

        if !bytes[..4].iter().all(|b| b.is_ascii_digit()) {
            return Err(ValidationError::InvalidPostalCode);
        }

        if !bytes[4..].iter().all(|b| b.is_ascii_alphabetic()) {
            return Err(ValidationError::InvalidPostalCode);
        }

        Ok(PostalCode(postal_code.to_ascii_uppercase()))
    }
}
