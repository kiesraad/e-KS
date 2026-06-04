use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    AppStore, ElectionConfig, OptionStringExt, TokenValue,
    common::{
        BsnOrNoneConfirmed, CountryCode, DateOfBirth, FullNameForm, Gender, Initials, LastName,
        LastNamePrefix, PlaceOfResidence,
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

impl PersonalDataFieldsForm {
    pub fn is_date_of_birth_very_old(&self) -> bool {
        self.date_of_birth
            .parse::<DateOfBirth>()
            .is_ok_and(|d| d.is_very_old())
    }
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
    /// Also checks person uniqueness and date of birth
    pub fn validate_create_with_checks(
        self,
        csrf_token: &TokenValue,
        store: &AppStore,
        election: &ElectionConfig,
    ) -> Result<Person, Box<FormData<Self>>> {
        let existing = store.get_persons();
        let existing_ref = existing.iter().collect();
        let person_result = self.clone().validate_create(csrf_token);
        let mut errors = self.uniqueness_errors(existing_ref);
        errors.extend(self.date_of_birth_check(election));

        self.merge_validation_results(person_result, errors, csrf_token)
    }

    /// Also checks date of birth
    pub fn validate_update_with_checks(
        self,
        current: &Person,
        csrf_token: &TokenValue,
        store: &AppStore,
        election: &ElectionConfig,
    ) -> Result<Person, Box<FormData<Self>>> {
        let existing = store.get_persons();
        let existing_without_current: Vec<&Person> =
            existing.iter().filter(|p| *p != current).collect();
        let person_result = self.clone().validate_update(current, csrf_token);
        let mut errors = self.uniqueness_errors(existing_without_current);
        errors.extend(self.date_of_birth_check(election));

        self.merge_validation_results(person_result, errors, csrf_token)
    }

    /// Validate that the BSN is unique OR the name (initials, prefix, lastname) is unique
    fn uniqueness_errors(&self, existing: Vec<&Person>) -> FieldErrors {
        let mut errors = Vec::new();

        // If a BSN is set, validate BSN uniqueness and skip name uniqueness checks
        if let Ok(BsnOrNoneConfirmed::Bsn(bsn)) =
            BsnOrNoneConfirmed::from_str(&self.personal_data.bsn)
        {
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

        if let Ok(last_name) = LastName::from_str(&self.name.last_name)
            && let Ok(initials) = Initials::from_str(&self.name.initials)
        {
            let last_name_prefix = if self.name.last_name_prefix.is_empty() {
                None
            } else {
                match LastNamePrefix::from_str(&self.name.last_name_prefix) {
                    Ok(prefix) => Some(prefix),
                    // Prefix validation will be handled by the outer validation
                    // we can ignore parsing errors here for the purpose of uniqueness checks
                    Err(_) => return Vec::new(),
                }
            };

            let has_duplicate_name = existing.iter().any(|p| {
                p.name.initials == initials
                    && p.name.last_name_prefix == last_name_prefix
                    && p.name.last_name == last_name
            });

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
        }

        errors
    }

    /// Validate that the date of birth is valid for the current election
    fn date_of_birth_check(&self, election: &ElectionConfig) -> FieldErrors {
        let mut errors = Vec::new();

        if let Ok(date) = DateOfBirth::from_str(&self.personal_data.date_of_birth)
            && NaiveDate::from(date) > election.eligible_date_of_birth()
        {
            errors.push((
                "personal_data.date_of_birth".to_string(),
                ValidationError::CandidateTooYoung,
            ));
        }

        errors
    }

    fn merge_validation_results(
        self,
        person_result: Result<Person, FormData<Self>>,
        additional_errors: FieldErrors,
        csrf_token: &TokenValue,
    ) -> Result<Person, Box<FormData<Self>>> {
        if additional_errors.is_empty() {
            return Ok(person_result?);
        }
        match person_result {
            Ok(_) => Err(Box::new(FormData::new_with_errors(
                self,
                csrf_token,
                additional_errors,
            ))),
            Err(form_data) => {
                let mut errors = form_data.errors();
                errors.extend(additional_errors);
                Err(Box::new(FormData::new_with_errors(
                    self, csrf_token, errors,
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        OptionAsStrExt,
        common::{DutchAddress, UtcDateTime},
        form::{ValidationError, generate_csrf_token},
        persons::PersonId,
        test_utils::{parse_country_code, parse_place_of_residence, sample_person_with},
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
        let csrf_token = generate_csrf_token();

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
            csrf_token: csrf_token.clone(),
        };

        let updated = form.validate_update(&current, &csrf_token).unwrap();

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
        let csrf_token = generate_csrf_token();
        let form = PersonalDataForm {
            name: FullNameForm {
                first_name: "🤔".to_string(),
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
            csrf_token: csrf_token.clone(),
        };

        let Err(data) = form.validate_create(&csrf_token) else {
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
        assert!(errors.contains(&("name.first_name".to_string(), ValidationError::InvalidValue)));
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

        let form = PersonalDataForm {
            name: FullNameForm {
                first_name: "".to_string(),
                last_name: "Klaas Smit".to_string(),
                last_name_prefix: "".to_string(),
                initials: "E.D.".to_string(),
            },
            personal_data: PersonalDataFieldsForm {
                gender: "m".to_string(),
                date_of_birth: "12-12-1970".to_string(),
                bsn: "".to_string(),
                place_of_residence: "Amsterdam".to_string(),
                country: "NL".to_string(),
            },
            csrf_token: generate_csrf_token(),
        };

        let errors = form.uniqueness_errors(vec![&existing]);

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

        let form = PersonalDataForm {
            name: FullNameForm {
                first_name: "".to_string(),
                last_name: "Other".to_string(),
                last_name_prefix: "".to_string(),
                initials: "E.D.".to_string(),
            },
            personal_data: PersonalDataFieldsForm {
                gender: "m".to_string(),
                date_of_birth: "12-12-1970".to_string(),
                bsn: "123456782".to_string(),
                place_of_residence: "Amsterdam".to_string(),
                country: "NL".to_string(),
            },
            csrf_token: generate_csrf_token(),
        };

        let errors = form.uniqueness_errors(vec![&existing]);

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

        let form = PersonalDataForm {
            name: FullNameForm {
                first_name: "".to_string(),
                last_name: "Klaas Smit".to_string(),
                last_name_prefix: "".to_string(),
                initials: "E.D.".to_string(),
            },
            personal_data: PersonalDataFieldsForm {
                gender: "m".to_string(),
                date_of_birth: "12-12-1970".to_string(),
                bsn: "111222333".to_string(),
                place_of_residence: "Amsterdam".to_string(),
                country: "NL".to_string(),
            },
            csrf_token: generate_csrf_token(),
        };

        let errors = form.uniqueness_errors(vec![&existing]);

        assert!(errors.is_empty());
    }

    #[test]
    fn candidate_too_young_errors() {
        let election = &ElectionConfig::EK27;

        let mut form = PersonalDataForm {
            name: FullNameForm {
                first_name: "".to_string(),
                last_name: "Klaas Smit".to_string(),
                last_name_prefix: "".to_string(),
                initials: "E.D.".to_string(),
            },
            personal_data: PersonalDataFieldsForm {
                gender: "m".to_string(),
                date_of_birth: "12-12-1970".to_string(),
                bsn: "111222333".to_string(),
                place_of_residence: "Amsterdam".to_string(),
                country: "NL".to_string(),
            },
            csrf_token: generate_csrf_token(),
        };

        let errors = form.date_of_birth_check(election);
        assert!(errors.is_empty());

        form.personal_data.date_of_birth = "12-12-2015".to_string();
        assert_eq!(
            form.date_of_birth_check(election),
            vec![(
                "personal_data.date_of_birth".to_string(),
                ValidationError::CandidateTooYoung
            )]
        );
    }

    #[test]
    fn validate_create_with_checks_combines_errors() {
        let store = AppStore::new_for_test();
        let csrf_token = generate_csrf_token();
        let form = PersonalDataForm {
            name: FullNameForm {
                first_name: "".to_string(),
                last_name: "Klaas Smit".to_string(),
                last_name_prefix: "invalid".to_string(),
                initials: "E.D.".to_string(),
            },
            personal_data: PersonalDataFieldsForm {
                gender: "m".to_string(),
                date_of_birth: "12-12-2015".to_string(),
                bsn: "111222333".to_string(),
                place_of_residence: "Amsterdam".to_string(),
                country: "NL".to_string(),
            },
            csrf_token: csrf_token.clone(),
        };

        let form_data = form
            .validate_create_with_checks(&csrf_token, &store, &ElectionConfig::EK27)
            .expect_err("form shouldn't validate");

        let errors = form_data.errors();

        assert_eq!(errors.len(), 2);
        assert!(errors.contains(&(
            "personal_data.date_of_birth".to_string(),
            ValidationError::CandidateTooYoung
        )));
        assert!(errors.contains(&(
            "name.last_name_prefix".to_string(),
            ValidationError::InvalidValue
        )));
    }
}
