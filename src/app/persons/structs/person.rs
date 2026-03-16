use serde::{Deserialize, Serialize};

use crate::{
    AppError, AppEvent, AppStore,
    common::{DutchAddress, FullName, Gender, UtcDateTime},
    core::AnyLocale,
    id_newtype,
    pagination::SortDirection,
    persons::{PersonSort, PersonalData, structs::person_sort::compare_persons},
};

id_newtype!(pub struct PersonId);

#[derive(Default, Debug, Serialize, Eq, PartialEq, Deserialize, Clone)]
pub struct Person {
    pub id: PersonId,
    pub name: FullName,
    pub personal_data: PersonalData,
    pub address: DutchAddress,
    pub representative: Representative,
    pub updated_at: UtcDateTime,
}

#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Representative {
    pub name: FullName,
    pub address: DutchAddress,
}

impl Representative {
    pub fn is_complete(&self) -> bool {
        self.name.is_complete() && self.address.is_complete()
    }
}

impl Person {
    pub async fn create_from_personal_data(
        store: &AppStore,
        name: FullName,
        personal_data: PersonalData,
    ) -> Result<Person, AppError> {
        let person_id = PersonId::new();

        store
            .update(AppEvent::CreatePersonPersonalData {
                person_id,
                name,
                personal_data,
            })
            .await?;

        store.get_person(person_id)
    }

    pub async fn update_personal_data(
        &self,
        store: &AppStore,
        name: FullName,
        personal_data: PersonalData,
    ) -> Result<Person, AppError> {
        store
            .update(AppEvent::UpdatePersonPersonalData {
                person_id: self.id,
                name: name.clone(),
                personal_data: personal_data.clone(),
            })
            .await?;

        store.get_person(self.id)
    }

    /// Returns the initials as printed on the candidate list,
    /// i.e., optionally with the first name and gender.
    ///
    /// **Examples:**
    /// - H. (Hubertus) (m)
    /// - H. (m)
    /// - H. (Hubertus)
    /// - H.
    pub fn initials_as_printed_on_list(&self, locale: AnyLocale) -> String {
        let mut initials = self.name.initials_with_first_name();
        if let Some(gender) = &self.personal_data.gender {
            initials.push_str(&format!(" ({})", &gender.abbreviation(locale)));
        }
        initials
    }

    pub fn lives_in_nl(&self) -> bool {
        match &self.personal_data.country {
            Some(country) => country.is_nl(),
            None => true, // Assume Dutch if no country is set
        }
    }

    pub fn gender_key(&self) -> &'static str {
        self.personal_data
            .gender
            .map(|g| match g {
                Gender::Male => "common.gender.male",
                Gender::Female => "common.gender.female",
            })
            .unwrap_or("")
    }

    pub fn is_personal_info_complete(&self) -> bool {
        self.name.is_complete()
            && self.personal_data.date_of_birth.is_some()
            && self.personal_data.bsn.is_some()
            && self.personal_data.place_of_residence.is_some()
            && self.personal_data.country.is_some()
    }

    pub fn is_representative_complete(&self) -> bool {
        if self.lives_in_nl() {
            return true;
        }

        self.representative.is_complete()
    }

    pub fn is_complete(&self) -> bool {
        self.is_personal_info_complete()
            && (!self.lives_in_nl() || self.address.is_complete())
            && (self.lives_in_nl() || self.is_representative_complete())
    }

    pub async fn create(&self, store: &AppStore) -> Result<(), AppError> {
        store.update(AppEvent::CreatePerson(self.clone())).await
    }

    pub async fn update(&self, store: &AppStore) -> Result<(), AppError> {
        store.update(AppEvent::UpdatePerson(self.clone())).await
    }

    pub async fn update_representative(
        &self,
        store: &AppStore,
        representative: Representative,
    ) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdatePersonRepresentative {
                person_id: self.id,
                representative,
            })
            .await
    }

    pub async fn update_address(
        &self,
        store: &AppStore,
        address: DutchAddress,
    ) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdatePersonAddress {
                person_id: self.id,
                address,
            })
            .await
    }

    pub async fn delete(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::DeletePerson { person_id: self.id })
            .await
    }

    pub fn list(
        store: &AppStore,
        limit: usize,
        offset: usize,
        sort_field: &PersonSort,
        sort_direction: &SortDirection,
    ) -> Result<Vec<Person>, AppError> {
        let mut persons = store.get_persons();
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
        common::BsnOrNoneConfirmed,
        pagination::SortDirection,
        persons::PersonSort,
        test_utils::{
            parse_country_code, parse_last_name_prefix, sample_person, sample_person_with,
            sample_person_with_last_name,
        },
    };

    #[tokio::test]
    async fn create_and_get_person() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let id = PersonId::new();
        let person = sample_person(id);

        person.create(&store).await?;

        let loaded = store.get_person(id)?;
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.name.last_name.to_string(), "Jansen");

        Ok(())
    }

    #[tokio::test]
    async fn update_person_overwrites_fields() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let id = PersonId::new();
        let mut person = sample_person(id);

        person.create(&store).await?;

        person.name.last_name = "Updated".parse().expect("last name");
        person.update(&store).await?;

        let updated = store.get_person(id)?;
        assert_eq!(updated.name.last_name.to_string(), "Updated");

        Ok(())
    }

    #[tokio::test]
    async fn remove_person_deletes_record() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let id = PersonId::new();
        let person = sample_person(id);

        person.create(&store).await?;
        person.delete(&store).await?;

        let missing = store.get_person(id);
        assert!(missing.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn update_address_overwrites_fields() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let id = PersonId::new();
        let mut person = sample_person(id);

        person.create(&store).await?;

        person.address.locality = Some("Nieuwegein".parse().expect("locality"));
        person.address.postal_code = Some("9999 ZZ".parse().expect("postal code"));
        person.address.house_number = Some("99".parse().expect("house number"));
        person.address.house_number_addition = None;
        person.address.street_name = Some("Nieuweweg".parse().expect("street name"));

        person
            .update_address(&store, person.address.clone())
            .await?;

        let updated = store.get_person(id)?;
        assert_eq!(
            updated.address.locality.as_deref().map(|v| v.to_string()),
            Some("Nieuwegein".to_string())
        );
        assert_eq!(
            updated.address.postal_code.unwrap(),
            "9999ZZ".parse().unwrap()
        );
        assert_eq!(
            updated
                .address
                .house_number
                .as_deref()
                .map(|v| v.to_string()),
            Some("99".to_string())
        );
        assert_eq!(updated.address.house_number_addition, None);
        assert_eq!(
            updated
                .address
                .street_name
                .as_deref()
                .map(|v| v.to_string()),
            Some("Nieuweweg".to_string())
        );

        Ok(())
    }

    #[tokio::test]
    async fn list_and_count_persons() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        sample_person_with_last_name(PersonId::new(), "Jansen")
            .create(&store)
            .await?;
        sample_person_with_last_name(PersonId::new(), "Bakker")
            .create(&store)
            .await?;

        let total = store.get_person_count();
        assert_eq!(total, 2);

        let persons = Person::list(&store, 10, 0, &PersonSort::LastName, &SortDirection::Asc)?;
        assert_eq!(persons.len(), 2);
        assert_eq!(persons[0].name.last_name.to_string(), "Bakker");

        Ok(())
    }

    #[test]
    fn last_name_formats_with_optional_prefix() {
        let mut person = sample_person_with(PersonId::new(), None, "Dijk", None, "A.B.");
        assert_eq!(person.name.last_name_with_prefix(), "Dijk");
        assert_eq!(person.name.last_name_with_prefix_appended(), "Dijk");

        person.name.last_name_prefix = Some(parse_last_name_prefix("van"));
        assert_eq!(person.name.last_name_with_prefix(), "van Dijk");
        assert_eq!(person.name.last_name_with_prefix_appended(), "Dijk, van");
    }

    #[test]
    fn display_name_shows_first_name_when_present() {
        let mut person =
            sample_person_with(PersonId::new(), Some("Anne"), "Dijk", Some("van"), "A.B.");
        assert_eq!(person.name.display(), "van Dijk, A.B. (Anne)");

        person.name.first_name = None;
        assert_eq!(person.name.display(), "van Dijk, A.B.");
    }

    #[test]
    fn lives_in_nl_defaults_to_true_and_accepts_variants() {
        let mut person = sample_person(PersonId::new());
        person.personal_data.country = None;
        assert!(person.lives_in_nl());

        person.personal_data.country = Some(parse_country_code("NL"));
        assert!(person.lives_in_nl());

        person.personal_data.country = Some(parse_country_code("BE"));
        assert!(!person.lives_in_nl());
    }

    #[test]
    fn gender_key_returns_translations_or_empty_keys() {
        let mut person = sample_person(PersonId::new());
        person.personal_data.gender = None;
        assert_eq!(person.gender_key(), "");

        person.personal_data.gender = Some(Gender::Male);
        assert_eq!(person.gender_key(), "common.gender.male");

        person.personal_data.gender = Some(Gender::Female);
        assert_eq!(person.gender_key(), "common.gender.female");
    }

    fn complete_address() -> DutchAddress {
        DutchAddress {
            locality: Some("Utrecht".parse().expect("locality")),
            postal_code: Some("1234 AB".parse().expect("postal code")),
            house_number: Some("10".parse().expect("house number")),
            house_number_addition: None,
            street_name: Some("Stationsstraat".parse().expect("street name")),
        }
    }

    fn complete_representative() -> Representative {
        Representative {
            name: FullName {
                first_name: Some("Anne".parse().expect("first name")),
                last_name: "Dijk".parse().expect("last name"),
                last_name_prefix: None,
                initials: "A.B.".parse().expect("initials"),
            },
            address: complete_address(),
        }
    }

    #[test]
    fn representative_is_complete_requires_name_and_address() {
        let mut representative = complete_representative();
        assert!(representative.is_complete());

        representative.address = DutchAddress::default();
        assert!(!representative.is_complete());
    }

    #[test]
    fn personal_info_complete_requires_core_fields() {
        let mut person = sample_person(PersonId::new());
        person.personal_data.bsn = None;
        assert!(!person.is_personal_info_complete());

        person.personal_data.bsn = Some(BsnOrNoneConfirmed::Bsn("999995972".parse().expect("bsn")));
        assert!(person.is_personal_info_complete());

        person.personal_data.date_of_birth = None;
        assert!(!person.is_personal_info_complete());
    }

    #[test]
    fn representative_complete_depends_on_country() {
        let mut person = sample_person(PersonId::new());
        assert!(person.is_representative_complete());

        person.personal_data.country = Some("BE".parse().expect("country code"));
        assert!(!person.is_representative_complete());

        person.representative = complete_representative();
        assert!(person.is_representative_complete());
    }

    #[test]
    fn person_complete_handles_dutch_and_non_dutch_requirements() {
        let mut dutch_person = sample_person(PersonId::new());
        dutch_person.personal_data.bsn =
            Some(BsnOrNoneConfirmed::Bsn("999995972".parse().expect("bsn")));
        assert!(dutch_person.is_complete());

        let mut non_dutch_person = sample_person(PersonId::new());
        non_dutch_person.personal_data.bsn =
            Some(BsnOrNoneConfirmed::Bsn("999995972".parse().expect("bsn")));
        non_dutch_person.personal_data.country = Some("BE".parse().expect("country code"));
        non_dutch_person.address = DutchAddress::default();
        assert!(!non_dutch_person.is_complete());

        non_dutch_person.representative = complete_representative();
        assert!(non_dutch_person.is_complete());
    }
}
