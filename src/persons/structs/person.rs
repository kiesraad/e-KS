use chrono::{DateTime, NaiveDate};
use serde::Serialize;
use sqlx::types::chrono::Utc;
use uuid::Uuid;

use crate::{constants::DEFAULT_DATE_TIME_FORMAT, t};

use super::Gender;

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct Person {
    pub id: Uuid,
    pub last_name: String,
    pub last_name_prefix: Option<String>,
    pub initials: String,
    pub first_name: Option<String>,
    pub gender: Option<Gender>,
    pub date_of_birth: Option<NaiveDate>,
    pub bsn: Option<String>,
    pub locality: Option<String>,
    pub postal_code: Option<String>,
    pub house_number: Option<String>,
    pub house_number_addition: Option<String>,
    pub street_name: Option<String>,
    pub is_dutch: Option<bool>,
    pub custom_country: Option<String>,
    pub custom_region: Option<String>,
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Person {
    /// Returns e.g. "van Dijk"
    pub fn last_name_with_prefix(&self) -> String {
        if let Some(prefix) = &self.last_name_prefix {
            format!("{} {}", prefix, self.last_name)
        } else {
            self.last_name.clone()
        }
    }

    /// Returns e.g. "Dijk, van"
    pub fn last_name_with_prefix_appended(&self) -> String {
        if let Some(prefix) = &self.last_name_prefix {
            format!("{}, {}", self.last_name, prefix)
        } else {
            self.last_name.clone()
        }
    }

    pub fn display_name(&self) -> String {
        if let Some(first_name) = &self.first_name {
            format!("{} {}", first_name, self.last_name_with_prefix())
        } else {
            format!("{} {}", self.initials, self.last_name_with_prefix())
        }
    }

    pub fn created(&self) -> String {
        self.created_at.format(DEFAULT_DATE_TIME_FORMAT).to_string()
    }

    pub fn updated(&self) -> String {
        self.updated_at.format(DEFAULT_DATE_TIME_FORMAT).to_string()
    }

    pub fn first_name_display(&self) -> String {
        self.first_name.clone().unwrap_or_default()
    }

    pub fn gender_key(&self) -> &[&'static str] {
        self.gender
            .map(|g| match g {
                Gender::Male => t!("gender.male"),
                Gender::Female => t!("gender.female"),
                Gender::X => t!("gender.x"),
            })
            .unwrap_or(&["", ""])
    }

    /// Make sure a person has either a Dutch address or an international address, but not both
    pub fn normalize_address(&mut self) {
        if self.is_dutch.is_none_or(|d| d) {
            // remove international address
            self.address_line_1 = None;
            self.address_line_2 = None;
            self.custom_region = None;
            self.custom_country = None;
        } else {
            // remove Dutch address
            self.postal_code = None;
            self.house_number = None;
            self.house_number_addition = None;
            self.street_name = None;
            self.locality = None;
        }
    }
}
