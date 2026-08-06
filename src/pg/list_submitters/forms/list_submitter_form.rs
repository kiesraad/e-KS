use crate::structs::common::{CountryCode, PostalCode};
use std::str::FromStr;

use crate::{
    common::{InternationalAddressForm, MinimalNameForm},
    form::{FieldErrors, FormData, MergeErrors},
    structs::list_submitters::{ListSubmitter, ListSubmitterData},
};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate, Clone)]
#[validate(target = "ListSubmitterData")]
#[serde(default)]
pub struct ListSubmitterForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: MinimalNameForm,
    #[validate(flatten)]
    #[serde(flatten)]
    pub address: InternationalAddressForm,
}

impl From<ListSubmitter> for ListSubmitterForm {
    fn from(value: ListSubmitter) -> Self {
        let value = ListSubmitterData::from(value);

        ListSubmitterForm {
            name: MinimalNameForm::from(value.name),
            address: InternationalAddressForm::from(value.address),
        }
    }
}

impl ListSubmitterForm {
    /// Also checks:
    /// if country code is NL -> postal code is a valid NL postal code
    pub fn validate_create_with_checks(self) -> Result<ListSubmitterData, Box<FormData<Self>>> {
        let submitter_result = self.clone().validate_create();
        let postal_code_errors = self.validate_postal_code();
        submitter_result.merge_errors(self, postal_code_errors)
    }

    /// Also checks:
    /// if country code is NL -> postal code is a valid NL postal code
    pub fn validate_update_with_checks(
        self,
        current: &ListSubmitterData,
    ) -> Result<ListSubmitterData, Box<FormData<Self>>> {
        let submitter = self.clone().validate_update(current);
        let postal_code_errors = self.validate_postal_code();
        submitter.merge_errors(self, postal_code_errors)
    }

    fn validate_postal_code(&self) -> FieldErrors {
        let mut errors = Vec::new();
        if let Ok(country) = CountryCode::from_str(&self.address.country)
            && country.is_nl()
            && !self.address.postal_code.is_empty()
            && let Err(error) = PostalCode::from_str(&self.address.postal_code)
        {
            errors.push(("address.postal_code".to_string(), error))
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{form::ValidationError, structs::common::Address};

    #[test]
    fn validate_create_uses_dutch_address_when_country_is_empty() {
        let form = ListSubmitterForm {
            name: MinimalNameForm {
                last_name: "Bos".to_string(),
                last_name_prefix: String::new(),
                initials: "E.F.".to_string(),
            },
            address: InternationalAddressForm {
                country: String::new(),
                locality: "Rotterdam".to_string(),
                state_or_province: String::new(),
                postal_code: "3011 CC".to_string(),
                house_number: "5".to_string(),
                house_number_addition: "B".to_string(),
                street_name: "Coolsingel".to_string(),
            },
        };

        let submitter: ListSubmitter = form
            .validate_create_with_checks()
            .expect("submitter")
            .into();

        assert!(matches!(submitter.address, Address::Dutch(_)));
    }

    #[test]
    fn validate_create_uses_international_address_when_country_is_foreign() {
        let form = ListSubmitterForm {
            name: MinimalNameForm {
                last_name: "Bos".to_string(),
                last_name_prefix: String::new(),
                initials: "E.F.".to_string(),
            },
            address: InternationalAddressForm {
                country: "BE".to_string(),
                locality: "Brussel".to_string(),
                state_or_province: "Brussels".to_string(),
                postal_code: "1000".to_string(),
                house_number: "1".to_string(),
                house_number_addition: String::new(),
                street_name: "Wetstraat".to_string(),
            },
        };

        let submitter: ListSubmitter = form
            .validate_create_with_checks()
            .expect("submitter")
            .into();

        match submitter.address {
            Address::International(address) => {
                assert_eq!(
                    address.country,
                    Some("BE".parse::<CountryCode>().expect("country"))
                );
                assert_eq!(
                    address
                        .state_or_province
                        .as_deref()
                        .map(ToString::to_string),
                    Some("Brussels".to_string())
                );
            }
            Address::Dutch(_) => panic!("expected international address"),
        }
    }

    #[test]
    fn validate_create_with_checks_validates_dutch_postal_code() {
        let form = ListSubmitterForm {
            name: MinimalNameForm {
                last_name: "Bos".to_string(),
                last_name_prefix: String::new(),
                initials: "E.F.".to_string(),
            },
            address: InternationalAddressForm {
                country: "NL".to_string(),
                locality: "Amsterdam".to_string(),
                state_or_province: String::new(),
                postal_code: "1000".to_string(),
                house_number: "1".to_string(),
                house_number_addition: String::new(),
                street_name: "Sample Street".to_string(),
            },
        };

        let form_data = form
            .validate_create_with_checks()
            .expect_err("Form shouldn't validate");

        let errors = form_data.errors();

        assert_eq!(errors.len(), 1);
        assert!(errors.contains(&(
            "address.postal_code".to_string(),
            ValidationError::InvalidPostalCode
        )));
    }

    #[test]
    fn validate_create_with_checks_combines_errors() {
        let form = ListSubmitterForm {
            name: MinimalNameForm {
                last_name: "Bos".to_string(),
                last_name_prefix: "invalid prefix".to_string(),
                initials: "E.F.".to_string(),
            },
            address: InternationalAddressForm {
                country: "NL".to_string(),
                locality: "Amsterdam".to_string(),
                state_or_province: String::new(),
                postal_code: "1000".to_string(),
                house_number: "1".to_string(),
                house_number_addition: String::new(),
                street_name: "Sample Street".to_string(),
            },
        };

        let form_data = form
            .validate_create_with_checks()
            .expect_err("Form shouldn't validate");

        let errors = form_data.errors();

        assert_eq!(errors.len(), 2);
        assert!(errors.contains(&(
            "address.postal_code".to_string(),
            ValidationError::InvalidPostalCode
        )));
        assert!(errors.contains(&(
            "name.last_name_prefix".to_string(),
            ValidationError::InvalidValue
        )));
    }
}
