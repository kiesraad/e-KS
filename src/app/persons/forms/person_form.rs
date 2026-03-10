use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    AppStore, CsrfTokens, OptionStringExt, TokenValue,
    common::{Bsn, CountryCode, Date, FirstName, FullNameForm, Gender, PlaceOfResidence},
    constants::DEFAULT_DATE_FORMAT,
    form::{FieldErrors, FormData, ValidationError},
    persons::Person,
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Person")]
#[serde(default)]
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
            bsn: person
                .bsn
                .as_ref()
                .map(|bsn| bsn.to_exposed_string())
                .unwrap_or_default(),
            no_bsn_confirmed: person.no_bsn_confirmed,
            place_of_residence: person.place_of_residence.to_string_or_default(),
            country_of_residence: person.country_of_residence.to_string_or_default(),
            csrf_token: Default::default(),
        }
    }
}

impl PersonForm {
    pub fn validate_create_unique(
        self,
        csrf_tokens: &CsrfTokens,
        store: &AppStore,
    ) -> Result<Person, Box<FormData<Self>>> {
        let existing = store.get_persons();
        let person = self.clone().validate_create(csrf_tokens)?;
        let errors = PersonForm::uniqueness_errors(&person, &existing);

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
        if let Some(bsn) = &person.bsn {
            if existing
                .iter()
                .any(|existing_person| existing_person.bsn.as_ref() == Some(bsn))
            {
                errors.push(("bsn".to_string(), ValidationError::BsnAlreadyExists));
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
        common::{DutchAddress, FullName, UtcDateTime},
        form::ValidationError,
        persons::PersonId,
    };

    fn base_person() -> Person {
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
            representative: Default::default(),
            updated_at: UtcDateTime::default(),
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
            updated
                .date_of_birth
                .map(|d| d.format(DEFAULT_DATE_FORMAT).to_string()),
            Some("01-02-2020".to_string())
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
        assert_eq!(person.gender_key(), "common.gender.male");

        person.first_name = None;
        assert_eq!(person.first_name.as_str_or_empty(), "");
        assert_eq!(person.display_name(), "E.D. Klaas Smit");
    }

    #[test]
    fn uniqueness_errors_for_duplicate_name_without_bsn() {
        let mut existing = base_person();
        existing.bsn = Some("123456782".parse().expect("bsn"));

        let mut incoming = base_person();
        incoming.bsn = None;

        let errors = PersonForm::uniqueness_errors(&incoming, &[existing]);

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
        let mut existing = base_person();
        existing.bsn = Some("123456782".parse().expect("bsn"));

        let mut incoming = base_person();
        incoming.name.last_name = "Other".parse().expect("last name");
        incoming.bsn = Some("123456782".parse().expect("bsn"));

        let errors = PersonForm::uniqueness_errors(&incoming, &[existing]);

        assert_eq!(
            errors,
            vec![("bsn".to_string(), ValidationError::BsnAlreadyExists)]
        );
    }

    #[test]
    fn uniqueness_allows_duplicate_name_with_unique_bsn() {
        let mut existing = base_person();
        existing.bsn = Some("123456782".parse().expect("bsn"));

        let mut incoming = base_person();
        incoming.bsn = Some("111222333".parse().expect("bsn"));

        let errors = PersonForm::uniqueness_errors(&incoming, &[existing]);

        assert!(errors.is_empty());
    }
}
