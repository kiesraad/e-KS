use std::str::FromStr;

use crate::{
    TokenValue,
    common::{CountryCode, InternationalAddressForm, MinimalNameForm, PostalCode},
    form::{FieldErrors, FormData},
    list_submitters::{ListSubmitter, ListSubmitterData},
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
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<ListSubmitter> for ListSubmitterForm {
    fn from(value: ListSubmitter) -> Self {
        let value = ListSubmitterData::from(value);

        ListSubmitterForm {
            name: MinimalNameForm::from(value.name),
            address: InternationalAddressForm::from(value.address),
            csrf_token: Default::default(),
        }
    }
}

impl ListSubmitterForm {
    /// Also checks:
    /// if country code is NL -> postal code is a valid NL postal code
    pub fn validate_create_with_checks(
        self,
        csrf_token: &TokenValue,
    ) -> Result<ListSubmitterData, Box<FormData<Self>>> {
        let submitter_result = self.clone().validate_create(csrf_token);
        let postal_code_errors = self.validate_postal_code();
        self.merge_validation_results(submitter_result, postal_code_errors, csrf_token)
    }

    /// Also checks:
    /// if country code is NL -> postal code is a valid NL postal code
    pub fn validate_update_with_checks(
        self,
        current: &ListSubmitterData,
        csrf_token: &TokenValue,
    ) -> Result<ListSubmitterData, Box<FormData<Self>>> {
        let submitter = self.clone().validate_update(current, csrf_token);
        let postal_code_errors = self.validate_postal_code();
        self.merge_validation_results(submitter, postal_code_errors, csrf_token)
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

    fn merge_validation_results(
        self,
        submitter_result: Result<ListSubmitterData, FormData<Self>>,
        postal_code_errors: FieldErrors,
        csrf_token: &TokenValue,
    ) -> Result<ListSubmitterData, Box<FormData<Self>>> {
        if postal_code_errors.is_empty() {
            return Ok(submitter_result?);
        }

        match submitter_result {
            Ok(_) => Err(Box::new(FormData::new_with_errors(
                self,
                csrf_token,
                postal_code_errors,
            ))),
            Err(form_data) => {
                let mut errors = form_data.errors();
                errors.extend(postal_code_errors);
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
        common::{Address, CountryCode},
        form::ValidationError,
    };

    #[test]
    fn validate_create_uses_dutch_address_when_country_is_empty() {
        let csrf_token = crate::form::generate_csrf_token();
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
            csrf_token: csrf_token.clone(),
        };

        let submitter: ListSubmitter = form
            .validate_create_with_checks(&csrf_token)
            .expect("submitter")
            .into();

        assert!(matches!(submitter.address, Address::Dutch(_)));
    }

    #[test]
    fn validate_create_uses_international_address_when_country_is_foreign() {
        let csrf_token = crate::form::generate_csrf_token();
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
            csrf_token: csrf_token.clone(),
        };

        let submitter: ListSubmitter = form
            .validate_create_with_checks(&csrf_token)
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
        let csrf_token = crate::form::generate_csrf_token();
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
            csrf_token: csrf_token.clone(),
        };

        let form_data = form
            .validate_create_with_checks(&csrf_token)
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
        let csrf_token = crate::form::generate_csrf_token();
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
            csrf_token: csrf_token.clone(),
        };

        let form_data = form
            .validate_create_with_checks(&csrf_token)
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
