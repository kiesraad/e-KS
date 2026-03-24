use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    AppStore, CsrfTokens, OptionStringExt, TokenValue,
    common::{
        BsnOrNoneConfirmed, CountryCode, DateOfBirth, FullNameForm, Gender, PlaceOfResidence,
    },
    constants::DEFAULT_DATE_FORMAT,
    form::{FieldErrors, FormData, ValidationError},
    persons::{Person, PersonalData},
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "PersonalData")]
#[serde(default)]
pub struct PersonalDataFieldsForm {
    #[validate(parse = "Gender", optional)]
    pub gender: String,
    #[validate(parse = "DateOfBirth", optional)]
    pub date_of_birth: String,
    #[validate(parse = "BsnOrNoneConfirmed", optional)]
    pub bsn: String,
    #[validate(parse = "PlaceOfResidence", optional)]
    pub place_of_residence: String,
    #[validate(parse = "CountryCode", optional)]
    pub country: String,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Person")]
#[serde(default)]
pub struct PersonalDataForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: FullNameForm,
    #[validate(flatten)]
    #[serde(flatten)]
    pub personal_data: PersonalDataFieldsForm,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<PersonalData> for PersonalDataFieldsForm {
    fn from(personal_data: PersonalData) -> Self {
        PersonalDataFieldsForm {
            gender: personal_data
                .gender
                .map(|g| g.to_string())
                .unwrap_or_default(),
            date_of_birth: personal_data
                .date_of_birth
                .map(|d| d.format(DEFAULT_DATE_FORMAT).to_string())
                .unwrap_or_default(),
            bsn: personal_data
                .bsn
                .map(|s| s.to_exposed_string())
                .unwrap_or_default(),
            place_of_residence: personal_data.place_of_residence.to_string_or_default(),
            country: personal_data.country.to_string_or_default(),
        }
    }
}

impl From<Person> for PersonalDataForm {
    fn from(person: Person) -> Self {
        PersonalDataForm {
            name: FullNameForm::from(person.name),
            personal_data: PersonalDataFieldsForm::from(person.personal_data),
            csrf_token: Default::default(),
        }
    }
}

impl PersonalDataForm {
    pub fn validate_create_unique(
        self,
        csrf_tokens: &CsrfTokens,
        store: &AppStore,
    ) -> Result<Person, Box<FormData<Self>>> {
        let existing = store.get_persons();
        let person = self.clone().validate_create(csrf_tokens)?;
        let errors = PersonalDataForm::uniqueness_errors(&person, &existing);

        if errors.is_empty() {
            Ok(person)
        } else {
            Err(Box::new(FormData::new_with_errors(
                self,
                csrf_tokens,
                errors,
            )))
        }
    }

    /// Validate that the BSN is unique OR the name (initials, prefix, lastname) is unique
    pub(crate) fn uniqueness_errors(person: &Person, existing: &[Person]) -> FieldErrors {
        let mut errors = Vec::new();

        // If a BSN is set, validate BSN uniqueness and skip name uniqueness checks
        if let Some(BsnOrNoneConfirmed::Bsn(bsn)) = &person.personal_data.bsn {
            if existing.iter().any(|existing_person| {
                existing_person.personal_data.bsn == Some(BsnOrNoneConfirmed::Bsn(bsn.clone()))
            }) {
                errors.push((
                    "personal_data.bsn".to_string(),
                    ValidationError::BsnAlreadyExists,
                ));
            }

            return errors;
        }

        let has_duplicate_name = existing
            .iter()
            .any(|existing_person| existing_person.name == person.name);

        if has_duplicate_name {
            errors.push((
                "name.initials".to_string(),
                ValidationError::NameAlreadyExists,
            ));
            errors.push((
                "name.last_name".to_string(),
                ValidationError::NameAlreadyExists,
            ));
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CsrfTokens, OptionAsStrExt,
        common::{DutchAddress, UtcDateTime},
        form::ValidationError,
        persons::PersonId,
        test_utils::{
            parse_country_code, parse_last_name, parse_place_of_residence, sample_person_with,
        },
    };

    #[test]
    fn personal_data_form_updates_existing_person_when_valid() {
        let mut current =
            sample_person_with(PersonId::new(), Some("Evert"), "Klaas Smit", None, "E.D.");
        current.personal_data.gender = Some(Gender::Female);
        current.personal_data.place_of_residence = Some(parse_place_of_residence("Waterdam"));
        current.personal_data.country = Some(parse_country_code("NL"));
        current.address = DutchAddress {
            locality: Some("Heemdamseburg".parse().expect("locality")),
            postal_code: Some("1234AB".parse().expect("postal code")),
            house_number: Some("10".parse().expect("house number")),
            house_number_addition: Some("B".parse().expect("house number addition")),
            street_name: Some("Spoorstraat".parse().expect("street name")),
        };
        current.updated_at = UtcDateTime::default();
        let tokens = CsrfTokens::default();

        let form = PersonalDataForm {
            name: FullNameForm {
                first_name: " Evert ".to_string(),
                last_name: "  Klaas Smit ".to_string(),
                last_name_prefix: "  van de ".to_string(),
                initials: "E.D.".to_string(),
            },
            personal_data: PersonalDataFieldsForm {
                gender: "male".to_string(),
                date_of_birth: "01-02-2020".to_string(),
                bsn: "none-confirmed".to_string(),
                place_of_residence: "Waterdam".to_string(),
                country: " nl ".to_string(),
            },
            csrf_token: tokens.issue().value,
        };

        let updated = form.validate_update(&current, &tokens).unwrap();

        assert_eq!(updated.id, current.id);
        assert_eq!(updated.personal_data.gender, Some(Gender::Male));
        assert_eq!(updated.name.last_name.to_string(), "Klaas Smit");
        assert_eq!(
            updated
                .name
                .last_name_prefix
                .as_deref()
                .map(|v| v.to_string()),
            Some("van de".to_string())
        );
        assert_eq!(
            updated.name.first_name.as_deref().map(|v| v.to_string()),
            Some("Evert".to_string())
        );
        assert_eq!(updated.name.initials.to_string(), "E.D.");
        assert_eq!(
            updated
                .personal_data
                .date_of_birth
                .map(|d| d.format(DEFAULT_DATE_FORMAT).to_string()),
            Some("01-02-2020".to_string())
        );
        assert_eq!(
            updated
                .personal_data
                .place_of_residence
                .as_deref()
                .map(|v| v.to_string()),
            Some("Waterdam".to_string())
        );
        assert_eq!(
            updated
                .personal_data
                .country
                .as_deref()
                .map(|v| v.to_string()),
            Some("NL".to_string())
        );
        assert_eq!(
            updated.address.locality.as_deref().map(|v| v.to_string()),
            Some("Heemdamseburg".to_string())
        );
        assert_eq!(
            updated.address.postal_code.unwrap(),
            "1234AB".parse().unwrap()
        );
        assert_eq!(
            updated
                .address
                .house_number
                .as_deref()
                .map(|v| v.to_string()),
            Some("10".to_string())
        );
        assert_eq!(
            updated
                .address
                .house_number_addition
                .as_deref()
                .map(|v| v.to_string()),
            Some("B".to_string())
        );
        assert_eq!(
            updated
                .address
                .street_name
                .as_deref()
                .map(|v| v.to_string()),
            Some("Spoorstraat".to_string())
        );
        assert!(updated.updated_at >= current.updated_at);
    }

    #[test]
    fn personal_data_form_collects_validation_errors() {
        let tokens = CsrfTokens::default();
        let form = PersonalDataForm {
            name: FullNameForm {
                first_name: " B ".to_string(),
                last_name: "de Bakker".to_string(),
                last_name_prefix: "Boris".to_string(),
                initials: "jd".to_string(),
            },
            personal_data: PersonalDataFieldsForm {
                gender: "invalid".to_string(),
                date_of_birth: "2020/01/01".to_string(),
                bsn: "".to_string(),
                place_of_residence: "x".to_string(),
                country: "xx".to_string(),
            },
            csrf_token: tokens.issue().value,
        };

        let Err(data) = form.validate_create(&tokens) else {
            panic!("expected validation errors");
        };

        let errors = data.errors();
        assert_eq!(errors.len(), 8);
        assert!(errors.contains(&(
            "personal_data.gender".to_string(),
            ValidationError::InvalidValue
        )));
        assert!(errors.contains(&(
            "name.last_name".to_string(),
            ValidationError::StartsWithLastNamePrefix
        )));
        assert!(errors.contains(&(
            "name.last_name_prefix".to_string(),
            ValidationError::InvalidValue
        )));
        assert!(errors.contains(&(
            "name.first_name".to_string(),
            ValidationError::ValueTooShort(1, 2)
        )));
        assert!(errors.contains(&("name.initials".to_string(), ValidationError::InvalidValue)));
        assert!(errors.contains(&(
            "personal_data.date_of_birth".to_string(),
            ValidationError::InvalidValue
        )));
        assert!(errors.contains(&(
            "personal_data.place_of_residence".to_string(),
            ValidationError::ValueTooShort(1, 2)
        )));
        assert!(errors.contains(&(
            "personal_data.country".to_string(),
            ValidationError::InvalidValue
        )));
    }

    #[test]
    fn display_helpers_behave_correctly() {
        let mut person =
            sample_person_with(PersonId::new(), Some("Evert"), "Klaas Smit", None, "E.D.");
        person.personal_data.gender = Some(Gender::Male);

        assert_eq!(person.name.display(), "Klaas Smit, E.D. (Evert)");
        assert_eq!(person.gender_key(), "common.gender.male");

        person.name.first_name = None;
        assert_eq!(person.name.first_name.as_str_or_empty(), "");
        assert_eq!(person.name.display(), "Klaas Smit, E.D.");
    }

    #[test]
    fn uniqueness_errors_for_duplicate_name_without_bsn() {
        let mut existing = sample_person_with(PersonId::new(), None, "Klaas Smit", None, "E.D.");
        existing.personal_data.bsn =
            Some(BsnOrNoneConfirmed::Bsn("123456782".parse().expect("bsn")));

        let mut incoming = sample_person_with(PersonId::new(), None, "Klaas Smit", None, "E.D.");
        incoming.personal_data.bsn = None;

        let errors = PersonalDataForm::uniqueness_errors(&incoming, &[existing]);

        assert!(errors.contains(&(
            "name.initials".to_string(),
            ValidationError::NameAlreadyExists
        )));
        assert!(errors.contains(&(
            "name.last_name".to_string(),
            ValidationError::NameAlreadyExists
        )));
    }

    #[test]
    fn uniqueness_errors_for_duplicate_bsn() {
        let mut existing = sample_person_with(PersonId::new(), None, "Klaas Smit", None, "E.D.");
        existing.personal_data.bsn =
            Some(BsnOrNoneConfirmed::Bsn("123456782".parse().expect("bsn")));

        let mut incoming = sample_person_with(PersonId::new(), None, "Klaas Smit", None, "E.D.");
        incoming.name.last_name = parse_last_name("Other");
        incoming.personal_data.bsn =
            Some(BsnOrNoneConfirmed::Bsn("123456782".parse().expect("bsn")));

        let errors = PersonalDataForm::uniqueness_errors(&incoming, &[existing]);

        assert_eq!(
            errors,
            vec![(
                "personal_data.bsn".to_string(),
                ValidationError::BsnAlreadyExists
            )]
        );
    }

    #[test]
    fn uniqueness_allows_duplicate_name_with_unique_bsn() {
        let mut existing = sample_person_with(PersonId::new(), None, "Klaas Smit", None, "E.D.");
        existing.personal_data.bsn =
            Some(BsnOrNoneConfirmed::Bsn("123456782".parse().expect("bsn")));

        let mut incoming = sample_person_with(PersonId::new(), None, "Klaas Smit", None, "E.D.");
        incoming.personal_data.bsn =
            Some(BsnOrNoneConfirmed::Bsn("111222333".parse().expect("bsn")));

        let errors = PersonalDataForm::uniqueness_errors(&incoming, &[existing]);

        assert!(errors.is_empty());
    }
}
