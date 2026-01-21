use chrono::{DateTime, NaiveDate};
use serde::Serialize;
use sqlx::types::chrono::Utc;

use crate::{id_newtype, persons::Gender, t};

id_newtype!(pub struct PersonId);

#[derive(Default, Debug, Serialize, Clone, sqlx::FromRow)]
pub struct Person {
    pub id: PersonId,
    pub last_name: String,
    pub last_name_prefix: Option<String>,
    pub initials: String,
    pub first_name: Option<String>,
    pub bsn: Option<String>,
    pub place_of_residence: Option<String>,
    pub country_of_residence: Option<String>,
    pub gender: Option<Gender>,
    pub date_of_birth: Option<NaiveDate>,
    pub locality: Option<String>,
    pub postal_code: Option<String>,
    pub house_number: Option<String>,
    pub house_number_addition: Option<String>,
    pub street_name: Option<String>,
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

    pub fn first_name_display(&self) -> String {
        self.first_name.clone().unwrap_or_default()
    }

    pub fn is_dutch(&self) -> bool {
        match &self.country_of_residence {
            Some(country) => {
                country.to_lowercase() == "netherlands" || country.to_lowercase() == "nederland"
            }
            None => true, // Assume Dutch if no country is set
        }
    }

    pub fn gender_key(&self) -> &[&'static str] {
        self.gender
            .map(|g| match g {
                Gender::Male => t!("gender.male"),
                Gender::Female => t!("gender.female"),
            })
            .unwrap_or(&["", ""])
    }
}
