use std::{io, str::FromStr};

use chrono::NaiveDate;
use csv::{ReaderBuilder, Trim};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError, AppStore, Bsn, CountryCode, Date, DutchAddress, FirstName, FullName, HouseNumber,
    Initials, LastName, Locality, PlaceOfResidence, PostalCode, StreetName, UtcDateTime,
    persons::{Gender, Person},
};

const PERSONS_CSV: &str = include_str!("persons.csv");

#[derive(Debug, Deserialize)]
struct PersonRecord {
    burgerservicenummer: String,
    geslacht: String,
    voornamen: String,
    geslachtsnaam: String,
    geboortedatum: String,
    straat: String,
    huisnummer: String,
    postcode: String,
    woonplaats: String,
}

impl PersonRecord {
    fn parse_value<T: FromStr>(value: &str, field: &str) -> Result<T, AppError> {
        value.parse::<T>().map_err(|_| {
            AppError::ServerError(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse {field}"),
            ))
        })
    }

    fn into_person(self) -> Result<Person, AppError> {
        let initials = self
            .voornamen
            .split_whitespace()
            .filter_map(|n| n.chars().next().map(|c| c.to_string()))
            .collect::<Vec<String>>()
            .join(".");

        let locality = if self.woonplaats.is_empty() {
            None
        } else {
            Some(Self::parse_value::<Locality>(&self.woonplaats, "locality")?)
        };

        let id = format!(
            "{}{}{}",
            self.burgerservicenummer, self.geslachtsnaam, initials
        );
        let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes());

        Ok(Person {
            id: uuid.into(),
            gender: match self.geslacht.as_str() {
                "M" => Some(Gender::Male),
                "V" => Some(Gender::Female),
                _ => None,
            },
            name: FullName {
                last_name: Self::parse_value::<LastName>(&self.geslachtsnaam, "last name")?,
                last_name_prefix: None,
                initials: Self::parse_value::<Initials>(&initials, "initials")?,
            },
            first_name: self
                .voornamen
                .split_whitespace()
                .next()
                .map(|s| Self::parse_value::<FirstName>(s, "first name"))
                .transpose()?,
            date_of_birth: NaiveDate::parse_from_str(&self.geboortedatum, "%Y%m%d")
                .ok()
                .map(Date::from),
            bsn: Self::parse_value::<Bsn>(&self.burgerservicenummer, "bsn").ok(),
            no_bsn_confirmed: false,
            place_of_residence: locality
                .as_deref()
                .map(|value| Self::parse_value::<PlaceOfResidence>(value, "place of residence"))
                .transpose()?,
            country_of_residence: Some(Self::parse_value::<CountryCode>("NL", "country code")?),
            address: DutchAddress {
                locality,
                postal_code: Some(self.postcode.parse::<PostalCode>().map_err(|_| {
                    AppError::ServerError(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Failed to parse postal code",
                    ))
                })?),
                house_number: Some(Self::parse_value::<HouseNumber>(
                    &self.huisnummer,
                    "house number",
                )?),
                house_number_addition: None,
                street_name: Some(Self::parse_value::<StreetName>(
                    &self.straat,
                    "street name",
                )?),
            },
            representative: Default::default(),
            created_at: UtcDateTime::now(),
            updated_at: UtcDateTime::now(),
        })
    }
}

pub async fn load(store: &AppStore) -> Result<(), AppError> {
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

        record.into_person()?.create(store).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{pagination::SortDirection, persons::PersonSort};
    use sqlx::PgPool;

    use super::*;

    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test]
    async fn test_load(pool: PgPool) {
        let store = AppStore::new(pool);
        load(&store).await.unwrap();
        let persons =
            crate::persons::Person::list(&store, 50, 0, &PersonSort::LastName, &SortDirection::Asc)
                .unwrap();

        assert_eq!(persons.len(), 50);
    }
}
