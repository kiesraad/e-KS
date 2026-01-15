use std::{io, str::FromStr};

use chrono::{NaiveDate, Utc};
use csv::{ReaderBuilder, Trim};
use serde::Deserialize;
use sqlx::PgConnection;

use crate::{
    AppError,
    constants::DEFAULT_DATE_FORMAT,
    persons::{
        self,
        structs::{Gender, Person},
    },
};

const PERSONS_CSV: &str = include_str!("persons.csv");

#[derive(Debug, Deserialize)]
struct PersonRecord {
    gender: Option<String>,
    last_name: String,
    last_name_prefix: Option<String>,
    first_name: Option<String>,
    initials: String,
    date_of_birth: String,
    locality: String,
    postal_code: String,
    house_number: String,
    house_number_addition: Option<String>,
    street_name: String,
}

impl PersonRecord {
    fn into_person(self) -> Result<Person, AppError> {
        Ok(Person {
            id: uuid::Uuid::new_v4(),
            gender: self.gender.and_then(|s| Gender::from_str(&s).ok()),
            last_name: self.last_name,
            last_name_prefix: self.last_name_prefix,
            first_name: self
                .first_name
                .and_then(|n| if n.is_empty() { None } else { Some(n) }),
            initials: self.initials,
            date_of_birth: NaiveDate::parse_from_str(&self.date_of_birth, DEFAULT_DATE_FORMAT).ok(),
            bsn: None,
            locality: Some(self.locality),
            postal_code: Some(self.postal_code),
            house_number: Some(self.house_number),
            house_number_addition: self
                .house_number_addition
                .and_then(|n| if n.is_empty() { None } else { Some(n) }),
            street_name: Some(self.street_name),
            is_dutch: Some(true),
            custom_country: None,
            custom_region: None,
            address_line_1: None,
            address_line_2: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

pub async fn load(conn: &mut PgConnection) -> Result<(), AppError> {
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(PERSONS_CSV.as_bytes());

    for record in reader.deserialize::<PersonRecord>() {
        let record = record.map_err(|err| {
            AppError::ServerError(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse CSV record: {err}"),
            ))
        })?;

        let person = record.into_person()?;
        persons::repository::create_person(conn, &person).await?;
    }

    Ok(())
}
