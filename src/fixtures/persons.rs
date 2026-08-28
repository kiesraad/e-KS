use std::{io, str::FromStr};

use chrono::NaiveDate;
use csv::{ReaderBuilder, Trim};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError, PgStore,
    structs::{
        common::{
            BsnOrNoneConfirmed, CountryCode, DateOfBirth, DutchAddress, FirstName, FullName,
            Gender, HouseNumber, Initials, LastName, Locality, PlaceOfResidence, PostalCode,
            StreetName,
        },
        persons::{Person, PersonalData},
    },
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
            .filter_map(|n| n.chars().next().map(|c| format!("{c}.")))
            .collect::<String>();

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
            name: self.full_name(&initials)?,
            personal_data: self.personal_data(locality.as_deref())?,
            address: self.address(locality)?,
            representative: Default::default(),
            ..Default::default()
        })
    }

    fn full_name(&self, initials: &str) -> Result<FullName, AppError> {
        Ok(FullName {
            first_name: self
                .voornamen
                .split_whitespace()
                .next()
                .map(|s| Self::parse_value::<FirstName>(s, "first name"))
                .transpose()?,
            last_name: Self::parse_value::<LastName>(&self.geslachtsnaam, "last name")?,
            last_name_prefix: None,
            initials: Self::parse_value::<Initials>(initials, "initials")?,
        })
    }

    fn personal_data(&self, locality: Option<&String>) -> Result<PersonalData, AppError> {
        Ok(PersonalData {
            gender: match self.geslacht.as_str() {
                "M" => Some(Gender::Male),
                "V" => Some(Gender::Female),
                _ => None,
            },
            date_of_birth: NaiveDate::parse_from_str(&self.geboortedatum, "%Y%m%d")
                .ok()
                .map(DateOfBirth::from),
            bsn: Self::parse_value::<BsnOrNoneConfirmed>(&self.burgerservicenummer, "bsn").ok(),
            place_of_residence: locality
                .map(|value| Self::parse_value::<PlaceOfResidence>(value, "place of residence"))
                .transpose()?,
            country: Some(Self::parse_value::<CountryCode>("NL", "country code")?),
        })
    }

    fn address(&self, locality: Option<Locality>) -> Result<DutchAddress, AppError> {
        Ok(DutchAddress {
            locality,
            postal_code: Some(Self::parse_value::<PostalCode>(
                &self.postcode,
                "postal code",
            )?),
            house_number: Some(Self::parse_value::<HouseNumber>(
                &self.huisnummer,
                "house number",
            )?),
            house_number_addition: None,
            street_name: Some(Self::parse_value::<StreetName>(
                &self.straat,
                "street name",
            )?),
            known_in_bag: None,
        })
    }
}

pub async fn load(store: &PgStore) -> Result<(), AppError> {
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

        // Skip records with errors, so we can generate documents
        // with the default set of fixtures
        if NaiveDate::parse_from_str(&record.geboortedatum, "%Y%m%d").is_err()
            || record.woonplaats.is_empty()
        {
            continue;
        }

        record.into_person()?.create(store).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{pagination::SortDirection, structs::persons::PersonSort};

    use super::*;

    #[tokio::test]
    async fn test_load() {
        let store = PgStore::new_for_test();
        load(&store).await.unwrap();

        let persons = crate::structs::persons::Person::list(
            &store,
            50,
            0,
            &PersonSort::LastName,
            &SortDirection::Asc,
        )
        .unwrap();

        assert_eq!(persons.len(), 50);
    }
}
