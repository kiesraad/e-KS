use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{constants::DEFAULT_DATE_FORMAT, form::ValidationError};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DateOfBirth(NaiveDate);

impl std::ops::Deref for DateOfBirth {
    type Target = chrono::NaiveDate;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for DateOfBirth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for DateOfBirth {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let naive_date = NaiveDate::parse_from_str(value, DEFAULT_DATE_FORMAT)
            .map_err(|_| ValidationError::InvalidValue)?;

        if naive_date > chrono::Utc::now().date_naive() {
            return Err(ValidationError::DateInFuture);
        }

        Ok(DateOfBirth(naive_date))
    }
}

impl From<NaiveDate> for DateOfBirth {
    fn from(value: NaiveDate) -> Self {
        Self(value)
    }
}

impl From<DateOfBirth> for NaiveDate {
    fn from(value: DateOfBirth) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_of_birth_cannot_be_in_the_future() {
        assert!(matches!(
            "01-01-9999".parse::<DateOfBirth>(),
            Err(ValidationError::DateInFuture),
        ));

        assert!("06-04-2001".parse::<DateOfBirth>().is_ok());
    }
}
