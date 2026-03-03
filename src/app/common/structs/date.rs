use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{constants::DEFAULT_DATE_FORMAT, form::ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Date(NaiveDate);

impl std::ops::Deref for Date {
    type Target = chrono::NaiveDate;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for Date {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        NaiveDate::parse_from_str(value, DEFAULT_DATE_FORMAT)
            .map(Date)
            .map_err(|_| ValidationError::InvalidValue)
    }
}

impl From<NaiveDate> for Date {
    fn from(value: NaiveDate) -> Self {
        Self(value)
    }
}

impl From<Date> for NaiveDate {
    fn from(value: Date) -> Self {
        value.0
    }
}
