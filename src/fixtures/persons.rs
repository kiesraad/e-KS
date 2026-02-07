use std::io;

use chrono::{NaiveDate, Utc};
use csv::{ReaderBuilder, Trim};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError, AppEvent, AppStore,
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
            Some(self.woonplaats)
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
            last_name: self.geslachtsnaam,
            last_name_prefix: None,
            first_name: self
                .voornamen
                .split_whitespace()
                .next()
                .map(|s| s.to_string()),
            initials,
            date_of_birth: NaiveDate::parse_from_str(&self.geboortedatum, "%Y%m%d").ok(),
            bsn: Some(self.burgerservicenummer),
            no_bsn_confirmed: false,
            place_of_residence: locality.clone(),
            country_of_residence: Some("NL".to_string()),
            locality,
            postal_code: Some(self.postcode),
            house_number: Some(self.huisnummer),
            house_number_addition: None,
            street_name: Some(self.straat),
            representative_initials: None,
            representative_last_name: None,
            representative_last_name_prefix: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
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

        let person = record.into_person()?;
        store.update(AppEvent::CreatePerson(person)).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{pagination::SortDirection, persons::PersonSort};
    use sqlx::PgPool;

    use super::*;

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
