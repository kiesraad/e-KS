use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    Bsn, CountryCode, Date, FirstName, FullNameForm, OptionStringExt, PlaceOfResidence, TokenValue,
    constants::DEFAULT_DATE_FORMAT,
    persons::{Gender, Person},
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Person")]
pub struct PersonForm {
    #[validate(parse = "Gender", optional)]
    pub gender: String,
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: FullNameForm,
    #[validate(parse = "FirstName", optional)]
    pub first_name: String,
    #[validate(parse = "Date", optional)]
    pub date_of_birth: String,
    #[validate(parse = "Bsn", optional)]
    pub bsn: String,
    #[serde(default)]
    pub no_bsn_confirmed: bool,
    #[validate(parse = "PlaceOfResidence", optional)]
    pub place_of_residence: String,
    #[validate(parse = "CountryCode", optional)]
    pub country_of_residence: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<Person> for PersonForm {
    fn from(person: Person) -> Self {
        PersonForm {
            gender: person.gender.map(|g| g.to_string()).unwrap_or_default(),
            name: FullNameForm::from(person.name),
            first_name: person.first_name.to_string_or_default(),
            date_of_birth: person
                .date_of_birth
                .map(|d| d.format(DEFAULT_DATE_FORMAT).to_string())
                .unwrap_or_default(),
            bsn: person.bsn.to_string_or_default(),
            no_bsn_confirmed: person.no_bsn_confirmed,
            place_of_residence: person.place_of_residence.to_string_or_default(),
            country_of_residence: person.country_of_residence.to_string_or_default(),
            csrf_token: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::{
        CsrfTokens, Date, DutchAddress, FullName, UtcDateTime, form::ValidationError,
        persons::PersonId,
    };

    fn base_person() -> Person {
        let timestamp = chrono::Utc
            .with_ymd_and_hms(2026, 1, 6, 7, 8, 9)
            .single()
            .unwrap();
        Person {
            id: PersonId::new(),
            gender: Some(Gender::Female),
            name: FullName {
                last_name: "Klaas Smit".parse().expect("last name"),
                last_name_prefix: None,
                initials: "E.D.".parse().expect("initials"),
            },
            first_name: Some("Evert".parse().expect("first name")),
            date_of_birth: None,
            bsn: None,
            no_bsn_confirmed: false,
            place_of_residence: Some("Waterdam".parse().expect("place of residence")),
            country_of_residence: Some("NL".parse().expect("country code")),
            address: DutchAddress {
                locality: Some("Heemdamseburg".parse().expect("locality")),
                postal_code: Some("1234AB".parse().expect("postal code")),
                house_number: Some("10".parse().expect("house number")),
                house_number_addition: Some("B".parse().expect("house number addition")),
                street_name: Some("Spoorstraat".parse().expect("street name")),
            },
            representative: FullName::default(),
            created_at: UtcDateTime::from(timestamp),
            updated_at: UtcDateTime::from(timestamp),
        }
    }

    #[test]
    fn person_form_updates_existing_person_when_valid() {
        let current = base_person();
        let tokens = CsrfTokens::default();

        let form = PersonForm {
            gender: "male".to_string(),
            name: FullNameForm {
                last_name: "  Klaas Smit ".to_string(),
                last_name_prefix: "  van de ".to_string(),
                initials: "E.D.".to_string(),
            },
            first_name: " Evert ".to_string(),
            date_of_birth: "01-02-2020".to_string(),
            bsn: "".to_string(),
            no_bsn_confirmed: true,
            place_of_residence: "Waterdam".to_string(),
            country_of_residence: " nl ".to_string(),
            csrf_token: tokens.issue().value,
        };

        let updated = form.validate_update(&current, &tokens).unwrap();

        assert_eq!(updated.id, current.id);
        assert_eq!(updated.gender, Some(Gender::Male));
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
            updated.first_name.as_deref().map(|v| v.to_string()),
            Some("Evert".to_string())
        );
        assert_eq!(updated.name.initials.to_string(), "E.D.");
        assert_eq!(
            updated.date_of_birth,
            Some("01-02-2020".parse::<Date>().unwrap())
        );
        assert_eq!(
            updated.place_of_residence.as_deref().map(|v| v.to_string()),
            Some("Waterdam".to_string())
        );
        assert_eq!(
            updated
                .country_of_residence
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
        assert_eq!(updated.created_at, current.created_at);
        assert!(updated.updated_at >= current.updated_at);
    }

    #[test]
    fn person_form_collects_validation_errors() {
        let tokens = CsrfTokens::default();
        let form = PersonForm {
            gender: "invalid".to_string(),
            name: FullNameForm {
                last_name: "de Bakker".to_string(),
                last_name_prefix: "Boris".to_string(),
                initials: "jd".to_string(),
            },
            first_name: " B ".to_string(),
            date_of_birth: "2020/01/01".to_string(),
            bsn: "".to_string(),
            no_bsn_confirmed: true,
            place_of_residence: "x".to_string(),
            country_of_residence: "xx".to_string(),
            csrf_token: tokens.issue().value,
        };

        let Err(data) = form.validate_create(&tokens) else {
            panic!("expected validation errors");
        };

        let errors = data.errors();
        assert_eq!(errors.len(), 8);
        assert!(errors.contains(&("gender".to_string(), ValidationError::InvalidValue)));
        assert!(errors.contains(&(
            "name.last_name".to_string(),
            ValidationError::StartsWithLastNamePrefix
        )));
        assert!(errors.contains(&(
            "name.last_name_prefix".to_string(),
            ValidationError::InvalidValue
        )));
        assert!(errors.contains(&(
            "first_name".to_string(),
            ValidationError::ValueTooShort(1, 2)
        )));
        assert!(errors.contains(&("name.initials".to_string(), ValidationError::InvalidValue)));
        assert!(errors.contains(&("date_of_birth".to_string(), ValidationError::InvalidValue)));
        assert!(errors.contains(&(
            "place_of_residence".to_string(),
            ValidationError::ValueTooShort(1, 2)
        )));
        assert!(errors.contains(&(
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
        assert_eq!(person.first_name.as_str_or_empty(), "");
        assert_eq!(person.display_name(), "E.D. Klaas Smit");
    }
}
