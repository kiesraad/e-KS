use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::{constants::DEFAULT_DATE_FORMAT, form::*};
use validate::Validate as ValidateDerive;

use super::{Gender, Person};

#[derive(Default, Serialize, Deserialize, Clone, Debug, ValidateDerive)]
#[validate(target = "Person", build = "PersonForm::build_person")]
#[serde(default)]
pub struct PersonForm {
    #[validate(parse = "Gender", optional)]
    pub gender: String,
    #[validate(with = "validate_length(2, 255)", with = "validate_teletex_chars()")]
    pub last_name: String,
    #[validate(
        with = "validate_length(2, 255)",
        with = "validate_teletex_chars()",
        optional
    )]
    pub first_name: String,
    #[validate(with = "validate_initials()")]
    pub initials: String,
    #[validate(
        parse_with = "chrono::NaiveDate::parse_from_str",
        format = DEFAULT_DATE_FORMAT,
        ty = "chrono::NaiveDate",
        optional
    )]
    pub date_of_birth: String,
    #[validate(with = "validate_length(2, 255)", optional)]
    pub locality: String,
    #[validate(with = "validate_length(2, 16)", optional)]
    pub postal_code: String,
    #[validate(with = "validate_length(1, 16)", optional)]
    pub house_number: String,
    #[validate(with = "validate_length(1, 16)", optional)]
    pub house_number_addition: String,
    #[validate(with = "validate_length(2, 255)", optional)]
    pub street_name: String,
    #[validate(csrf)]
    pub csrf_token: String,
}

impl From<Person> for PersonForm {
    fn from(person: Person) -> Self {
        PersonForm {
            gender: person.gender.map(|g| g.to_string()).unwrap_or_default(),
            last_name: person.last_name,
            first_name: person.first_name.unwrap_or_default(),
            initials: person.initials,
            date_of_birth: person
                .date_of_birth
                .map(|d| d.format(DEFAULT_DATE_FORMAT).to_string())
                .unwrap_or_default(),
            locality: person.locality.unwrap_or_default(),
            postal_code: person.postal_code.unwrap_or_default(),
            house_number: person.house_number.unwrap_or_default(),
            house_number_addition: person.house_number_addition.unwrap_or_default(),
            street_name: person.street_name.unwrap_or_default(),
            csrf_token: String::new(),
        }
    }
}

impl WithCsrfToken for PersonForm {
    fn with_csrf_token(self, csrf_token: CsrfToken) -> Self {
        PersonForm {
            csrf_token: csrf_token.value,
            ..self
        }
    }
}

impl PersonForm {
    fn build_person(validated: PersonFormValidated, current: Option<&Person>) -> Person {
        if let Some(current_person) = current {
            Person {
                gender: validated.gender,
                last_name: validated.last_name,
                first_name: validated.first_name,
                initials: validated.initials,
                date_of_birth: validated.date_of_birth,
                locality: validated.locality,
                postal_code: validated.postal_code,
                house_number: validated.house_number,
                house_number_addition: validated.house_number_addition,
                street_name: validated.street_name,
                ..current_person.clone()
            }
        } else {
            Person {
                id: Uuid::new_v4(),
                gender: validated.gender,
                last_name: validated.last_name,
                first_name: validated.first_name,
                initials: validated.initials,
                date_of_birth: validated.date_of_birth,
                locality: validated.locality,
                postal_code: validated.postal_code,
                house_number: validated.house_number,
                house_number_addition: validated.house_number_addition,
                street_name: validated.street_name,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::{
        CsrfTokens,
        form::{Validate, ValidationError},
    };
    use chrono::NaiveDate;

    fn base_person() -> Person {
        let timestamp = chrono::Utc
            .with_ymd_and_hms(2024, 5, 6, 7, 8, 9)
            .single()
            .unwrap();
        Person {
            id: Uuid::nil(),
            gender: Some(Gender::Female),
            last_name: "Old".to_string(),
            first_name: Some("Existing".to_string()),
            initials: "E.X.".to_string(),
            date_of_birth: None,
            locality: Some("Oldtown".to_string()),
            postal_code: Some("1234AB".to_string()),
            house_number: Some("10".to_string()),
            house_number_addition: Some("B".to_string()),
            street_name: Some("Old Street".to_string()),
            created_at: timestamp,
            updated_at: timestamp,
        }
    }

    #[test]
    fn person_form_updates_existing_person_when_valid() {
        let current = base_person();
        let tokens = CsrfTokens::default();

        let form = PersonForm {
            gender: "male".to_string(),
            last_name: "  Doe ".to_string(),
            first_name: " John ".to_string(),
            initials: "J.D.".to_string(),
            date_of_birth: "01-02-2020".to_string(),
            locality: " Utrecht ".to_string(),
            postal_code: " 1234 AB ".to_string(),
            house_number: " 12 ".to_string(),
            house_number_addition: " B ".to_string(),
            street_name: " Stationsstraat ".to_string(),
            csrf_token: tokens.issue().value,
        };

        let updated = form.validate(Some(&current), &tokens).unwrap();

        assert_eq!(updated.id, current.id);
        assert_eq!(updated.gender, Some(Gender::Male));
        assert_eq!(updated.last_name, "Doe");
        assert_eq!(updated.first_name, Some("John".to_string()));
        assert_eq!(updated.initials, "J.D.");
        assert_eq!(
            updated.date_of_birth,
            Some(NaiveDate::from_ymd_opt(2020, 2, 1).unwrap())
        );
        assert_eq!(updated.locality, Some("Utrecht".to_string()));
        assert_eq!(updated.postal_code, Some("1234 AB".to_string()));
        assert_eq!(updated.house_number, Some("12".to_string()));
        assert_eq!(updated.house_number_addition, Some("B".to_string()));
        assert_eq!(updated.street_name, Some("Stationsstraat".to_string()));
        assert_eq!(updated.created_at, current.created_at);
        assert_eq!(updated.updated_at, current.updated_at);
    }

    #[test]
    fn person_form_collects_validation_errors() {
        let tokens = CsrfTokens::default();
        let form = PersonForm {
            gender: "invalid".to_string(),
            last_name: "X".to_string(),
            first_name: " B ".to_string(),
            initials: "jd".to_string(),
            date_of_birth: "2020/01/01".to_string(),
            locality: "y".to_string(),
            postal_code: "x".to_string(),
            house_number: "".to_string(),
            house_number_addition: " ".to_string(),
            street_name: "z".to_string(),
            csrf_token: tokens.issue().value,
        };

        let Err(data) = form.validate(None, &tokens) else {
            panic!("expected validation errors");
        };

        assert_eq!(data.errors().len(), 9);
        assert!(
            data.errors()
                .contains(&("gender".to_string(), ValidationError::InvalidValue))
        );
        assert!(data.errors().contains(&(
            "last_name".to_string(),
            ValidationError::ValueTooShort(1, 2)
        )));
        assert!(data.errors().contains(&(
            "first_name".to_string(),
            ValidationError::ValueTooShort(1, 2)
        )));
        assert!(
            data.errors()
                .contains(&("initials".to_string(), ValidationError::InvalidValue))
        );
        assert!(
            data.errors()
                .contains(&("date_of_birth".to_string(), ValidationError::InvalidValue))
        );
        assert!(
            data.errors()
                .contains(&("locality".to_string(), ValidationError::ValueTooShort(1, 2)))
        );
        assert!(data.errors().contains(&(
            "postal_code".to_string(),
            ValidationError::ValueTooShort(1, 2)
        )));
        assert!(data.errors().contains(&(
            "house_number_addition".to_string(),
            ValidationError::ValueShouldNotBeEmpty
        )));
        assert!(data.errors().contains(&(
            "street_name".to_string(),
            ValidationError::ValueTooShort(1, 2)
        )));
    }

    #[test]
    fn display_helpers_behave_correctly() {
        let mut person = base_person();
        person.gender = Some(Gender::Male);

        assert_eq!(person.display_name(), "Existing Old");
        assert_eq!(person.gender_key(), &["Male", "man"]);
        assert_eq!(person.created(), "06-05-2024 07:08:09");
        assert_eq!(person.updated(), "06-05-2024 07:08:09");

        person.first_name = None;
        assert_eq!(person.first_name_display(), "");
        assert_eq!(person.display_name(), "Old");
    }
}
