use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;

use crate::{
    AppError, AppEvent, AppStore, id_newtype,
    pagination::SortDirection,
    persons::{Gender, PersonSort, structs::person_sort::compare_persons},
};

id_newtype!(pub struct PersonId);

#[derive(Default, Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Person {
    pub id: PersonId,

    pub last_name: String,
    pub last_name_prefix: Option<String>,
    pub initials: String,

    pub first_name: Option<String>,
    pub gender: Option<Gender>,

    pub bsn: Option<String>,
    pub no_bsn_confirmed: bool,
    pub date_of_birth: Option<NaiveDate>,

    pub place_of_residence: Option<String>,
    pub country_of_residence: Option<String>,

    pub street_name: Option<String>,
    pub house_number: Option<String>,
    pub house_number_addition: Option<String>,
    pub locality: Option<String>,
    pub postal_code: Option<String>,

    pub representative_last_name: Option<String>,
    pub representative_last_name_prefix: Option<String>,
    pub representative_initials: Option<String>,

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
            Some(country) => country == "NL",
            None => true, // Assume Dutch if no country is set
        }
    }

    pub fn gender_key(&self) -> &'static str {
        self.gender
            .map(|g| match g {
                Gender::Male => "gender.male",
                Gender::Female => "gender.female",
            })
            .unwrap_or("")
    }

    pub fn is_personal_info_complete(&self) -> bool {
        !self.initials.is_empty()
            && !self.last_name.is_empty()
            && self.date_of_birth.is_some()
            && self.bsn.is_some()
            && self.place_of_residence.is_some()
            && self.country_of_residence.is_some()
    }

    pub fn is_address_complete(&self) -> bool {
        self.street_name.is_some()
            && self.house_number.is_some()
            && self.postal_code.is_some()
            && self.locality.is_some()
    }

    pub fn is_representative_complete(&self) -> bool {
        if self.is_dutch() {
            return true;
        }

        self.is_address_complete()
            && !self
                .representative_initials
                .as_deref()
                .unwrap_or("")
                .is_empty()
            && !self
                .representative_last_name
                .as_deref()
                .unwrap_or("")
                .is_empty()
    }

    pub fn is_complete(&self) -> bool {
        self.is_personal_info_complete()
            && self.is_address_complete()
            && self.is_representative_complete()
    }

    pub async fn create(&self, store: &AppStore) -> Result<(), AppError> {
        store.update(AppEvent::CreatePerson(self.clone())).await
    }

    pub async fn update(&self, store: &AppStore) -> Result<(), AppError> {
        store.update(AppEvent::UpdatePerson(self.clone())).await
    }

    pub async fn update_address(&self, store: &AppStore) -> Result<(), AppError> {
        let existing = store.get_person(self.id)?;

        let updated = Person {
            locality: self.locality.clone(),
            postal_code: self.postal_code.clone(),
            house_number: self.house_number.clone(),
            house_number_addition: self.house_number_addition.clone(),
            street_name: self.street_name.clone(),
            updated_at: Utc::now(),
            ..existing
        };

        store.update(AppEvent::UpdatePerson(updated.clone())).await
    }

    pub async fn delete(&self, store: &AppStore) -> Result<(), AppError> {
        store.update(AppEvent::DeletePerson(self.id)).await
    }

    pub async fn delete_by_id(store: &AppStore, person_id: PersonId) -> Result<(), AppError> {
        store.update(AppEvent::DeletePerson(person_id)).await
    }

    pub fn list(
        store: &AppStore,
        limit: usize,
        offset: usize,
        sort_field: &PersonSort,
        sort_direction: &SortDirection,
    ) -> Result<Vec<Person>, AppError> {
        let mut persons = store.get_persons()?;
        persons.sort_by(|a, b| compare_persons(a, b, sort_field));

        if matches!(sort_direction, SortDirection::Desc) {
            persons.reverse();
        }

        let offset = offset.max(0);
        let limit = limit.max(0);

        Ok(persons.into_iter().skip(offset).take(limit).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore,
        pagination::SortDirection,
        persons::PersonSort,
        test_utils::{sample_person, sample_person_with_last_name},
    };
    use sqlx::{PgPool, types::chrono::Utc};

    fn base_person() -> Person {
        Person {
            id: PersonId::new(),
            last_name: "Dijk".to_string(),
            last_name_prefix: None,
            initials: "A.B.".to_string(),
            first_name: None,
            bsn: None,
            no_bsn_confirmed: false,
            place_of_residence: None,
            country_of_residence: None,
            gender: None,
            date_of_birth: None,
            locality: None,
            postal_code: None,
            house_number: None,
            house_number_addition: None,
            street_name: None,
            representative_last_name: None,
            representative_last_name_prefix: None,
            representative_initials: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[sqlx::test]
    async fn create_and_get_person(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let id = PersonId::new();
        let person = sample_person(id);

        person.create(&store).await?;

        let loaded = store.get_person(id)?;
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.last_name, "Jansen");

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_overwrites_fields(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let id = PersonId::new();
        let mut person = sample_person(id);

        person.create(&store).await?;

        person.last_name = "Updated".to_string();
        person.update(&store).await?;

        let updated = store.get_person(id)?;
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn remove_person_deletes_record(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let id = PersonId::new();
        let person = sample_person(id);

        person.create(&store).await?;
        person.delete(&store).await?;

        let missing = store.get_person(id);
        assert!(missing.is_err());

        Ok(())
    }

    #[sqlx::test]
    async fn update_address_overwrites_fields(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        let id = PersonId::new();
        let mut person = sample_person(id);

        person.create(&store).await?;

        person.locality = Some("Nieuwegein".to_string());
        person.postal_code = Some("9999 ZZ".to_string());
        person.house_number = Some("99".to_string());
        person.house_number_addition = None;
        person.street_name = Some("Nieuweweg".to_string());

        person.update_address(&store).await?;

        let updated = store.get_person(id)?;
        assert_eq!(updated.locality, Some("Nieuwegein".to_string()));
        assert_eq!(updated.postal_code, Some("9999 ZZ".to_string()));
        assert_eq!(updated.house_number, Some("99".to_string()));
        assert_eq!(updated.house_number_addition, None);
        assert_eq!(updated.street_name, Some("Nieuweweg".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn list_and_count_persons(pool: PgPool) -> Result<(), AppError> {
        let store = AppStore::new(pool);
        sample_person_with_last_name(PersonId::new(), "Jansen")
            .create(&store)
            .await?;
        sample_person_with_last_name(PersonId::new(), "Bakker")
            .create(&store)
            .await?;

        let total = store.get_person_count()?;
        assert_eq!(total, 2);

        let persons = Person::list(&store, 10, 0, &PersonSort::LastName, &SortDirection::Asc)?;
        assert_eq!(persons.len(), 2);
        assert_eq!(persons[0].last_name, "Bakker");

        Ok(())
    }

    #[test]
    fn last_name_formats_with_optional_prefix() {
        let mut person = base_person();
        assert_eq!(person.last_name_with_prefix(), "Dijk");
        assert_eq!(person.last_name_with_prefix_appended(), "Dijk");

        person.last_name_prefix = Some("van".to_string());
        assert_eq!(person.last_name_with_prefix(), "van Dijk");
        assert_eq!(person.last_name_with_prefix_appended(), "Dijk, van");
    }

    #[test]
    fn display_name_prefers_first_name_over_initials() {
        let mut person = base_person();
        person.last_name_prefix = Some("van".to_string());
        person.first_name = Some("Anne".to_string());
        assert_eq!(person.display_name(), "Anne van Dijk");

        person.first_name = None;
        assert_eq!(person.display_name(), "A.B. van Dijk");
    }

    #[test]
    fn first_name_display_falls_back_to_empty_string() {
        let mut person = base_person();
        assert_eq!(person.first_name_display(), "");

        person.first_name = Some("Henk".to_string());
        assert_eq!(person.first_name_display(), "Henk");
    }

    #[test]
    fn is_dutch_defaults_to_true_and_accepts_variants() {
        let mut person = base_person();
        assert!(person.is_dutch());

        person.country_of_residence = Some("NL".to_string());
        assert!(person.is_dutch());

        person.country_of_residence = Some("BE".to_string());
        assert!(!person.is_dutch());
    }

    #[test]
    fn gender_key_returns_translations_or_empty_keys() {
        let mut person = base_person();
        assert_eq!(person.gender_key(), "");

        person.gender = Some(Gender::Male);
        assert_eq!(person.gender_key(), "gender.male");

        person.gender = Some(Gender::Female);
        assert_eq!(person.gender_key(), "gender.female");
    }
}
