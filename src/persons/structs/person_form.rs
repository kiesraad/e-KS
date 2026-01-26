use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    CsrfToken, TokenValue,
    constants::DEFAULT_DATE_FORMAT,
    form::{
        WithCsrfToken, validate_country_code, validate_eleven_check, validate_initials,
        validate_last_name_prefix, validate_length, validate_no_last_name_prefix,
        validate_teletex_chars,
    },
    persons::{Gender, Person, structs::person::PersonId},
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Person", build = "PersonForm::build_person")]
#[serde(default)]
pub struct PersonForm {
    #[validate(parse = "Gender", optional)]
    pub gender: String,
    #[validate(
        with = "validate_length(2, 255)",
        with = "validate_teletex_chars()",
        with = "validate_no_last_name_prefix()"
    )]
    pub last_name: String,
    #[validate(with = "validate_last_name_prefix()", optional)]
    pub last_name_prefix: String,
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
    #[validate(with = "validate_eleven_check()", optional)]
    pub bsn: String,
    #[validate(with = "validate_length(2, 255)", optional)]
    pub place_of_residence: String,
    #[validate(with = "validate_country_code()", optional)]
    pub country_of_residence: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<Person> for PersonForm {
    fn from(person: Person) -> Self {
        PersonForm {
            gender: person.gender.map(|g| g.to_string()).unwrap_or_default(),
            last_name: person.last_name,
            last_name_prefix: person.last_name_prefix.unwrap_or_default(),
            first_name: person.first_name.unwrap_or_default(),
            initials: person.initials,
            date_of_birth: person
                .date_of_birth
                .map(|d| d.format(DEFAULT_DATE_FORMAT).to_string())
                .unwrap_or_default(),
            bsn: person.bsn.unwrap_or_default(),
            place_of_residence: person.place_of_residence.unwrap_or_default(),
            country_of_residence: person.country_of_residence.unwrap_or_default(),
            csrf_token: Default::default(),
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
    fn build_person(validated: PersonFormValidated, current: Option<Person>) -> Person {
        if let Some(current_person) = current {
            Person {
                gender: validated.gender,
                last_name: validated.last_name,
                last_name_prefix: validated.last_name_prefix,
                first_name: validated.first_name,
                initials: validated.initials,
                date_of_birth: validated.date_of_birth,
                bsn: validated.bsn,
                place_of_residence: validated.place_of_residence,
                country_of_residence: validated.country_of_residence,
                ..current_person
            }
        } else {
            Person {
                id: PersonId::new(),
                gender: validated.gender,
                last_name: validated.last_name,
                last_name_prefix: validated.last_name_prefix,
                first_name: validated.first_name,
                initials: validated.initials,
                date_of_birth: validated.date_of_birth,
                bsn: validated.bsn,
                place_of_residence: validated.place_of_residence,
                country_of_residence: validated.country_of_residence,
                locality: None,
                postal_code: None,
                house_number: None,
                house_number_addition: None,
                street_name: None,
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
            id: PersonId::new(),
            gender: Some(Gender::Female),
            last_name: "Klaas Smit".to_string(),
            last_name_prefix: None,
            first_name: Some("Evert".to_string()),
            initials: "E.D.".to_string(),
            date_of_birth: None,
            bsn: None,
            place_of_residence: Some("Waterdam".to_string()),
            country_of_residence: Some("NL".to_string()),
            locality: Some("Heemdamseburg".to_string()),
            postal_code: Some("1234AB".to_string()),
            house_number: Some("10".to_string()),
            house_number_addition: Some("B".to_string()),
            street_name: Some("Spoorstraat".to_string()),
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
            last_name: "  Klaas Smit ".to_string(),
            last_name_prefix: "  van de ".to_string(),
            first_name: " Evert ".to_string(),
            initials: "E.D.".to_string(),
            date_of_birth: "01-02-2020".to_string(),
            bsn: "".to_string(),
            place_of_residence: "Waterdam".to_string(),
            country_of_residence: " nl ".to_string(),
            csrf_token: tokens.issue().value,
        };

        let updated = form.validate_update(current.clone(), &tokens).unwrap();

        assert_eq!(updated.id, current.id);
        assert_eq!(updated.gender, Some(Gender::Male));
        assert_eq!(updated.last_name, "Klaas Smit");
        assert_eq!(updated.last_name_prefix, Some("van de".to_string()));
        assert_eq!(updated.first_name, Some("Evert".to_string()));
        assert_eq!(updated.initials, "E.D.");
        assert_eq!(
            updated.date_of_birth,
            Some(NaiveDate::from_ymd_opt(2020, 2, 1).unwrap())
        );
        assert_eq!(updated.place_of_residence, Some("Waterdam".to_string()));
        assert_eq!(updated.country_of_residence, Some("NL".to_string()));
        assert_eq!(updated.locality, Some("Heemdamseburg".to_string()));
        assert_eq!(updated.postal_code, Some("1234AB".to_string()));
        assert_eq!(updated.house_number, Some("10".to_string()));
        assert_eq!(updated.house_number_addition, Some("B".to_string()));
        assert_eq!(updated.street_name, Some("Spoorstraat".to_string()));
        assert_eq!(updated.created_at, current.created_at);
        assert_eq!(updated.updated_at, current.updated_at);
    }

    #[test]
    fn person_form_collects_validation_errors() {
        let tokens = CsrfTokens::default();
        let form = PersonForm {
            gender: "invalid".to_string(),
            last_name: "de Bakker".to_string(),
            last_name_prefix: "Boris".to_string(),
            first_name: " B ".to_string(),
            initials: "jd".to_string(),
            date_of_birth: "2020/01/01".to_string(),
            bsn: "".to_string(),
            place_of_residence: "x".to_string(),
            country_of_residence: "xx".to_string(),
            csrf_token: tokens.issue().value,
        };

        let Err(data) = form.validate_create(&tokens) else {
            panic!("expected validation errors");
        };

        assert_eq!(data.errors().len(), 8);
        assert!(
            data.errors()
                .contains(&("gender".to_string(), ValidationError::InvalidValue))
        );
        assert!(data.errors().contains(&(
            "last_name".to_string(),
            ValidationError::StartsWithLastNamePrefix
        )));
        assert!(data.errors().contains(&(
            "last_name_prefix".to_string(),
            ValidationError::InvalidValue
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
        assert!(data.errors().contains(&(
            "place_of_residence".to_string(),
            ValidationError::ValueTooShort(1, 2)
        )));
        assert!(data.errors().contains(&(
            "country_of_residence".to_string(),
            ValidationError::InvalidValue
        )));
    }

    #[test]
    fn display_helpers_behave_correctly() {
        let mut person = base_person();
        person.gender = Some(Gender::Male);

        assert_eq!(person.display_name(), "Evert Klaas Smit");
        assert_eq!(person.gender_key(), "gender.male");

        person.first_name = None;
        assert_eq!(person.first_name_display(), "");
        assert_eq!(person.display_name(), "E.D. Klaas Smit");
    }
}
