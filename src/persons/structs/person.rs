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
    pub initials: String,
    pub first_name: Option<String>,
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
    pub fn display_name(&self) -> String {
        if let Some(first_name) = &self.first_name {
            format!("{} {}", first_name, self.last_name)
        } else {
            self.last_name.clone()
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
}
