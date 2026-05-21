use std::str::FromStr;

use crate::{
    TokenValue,
    common::{CountryCode, InternationalAddressForm, PostalCode},
    form::FormData,
    list_submitters::{ListSubmitter, ListSubmitterData, SubmitterNameForm},
};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate, Clone)]
#[validate(target = "ListSubmitterData")]
#[serde(default)]
pub struct ListSubmitterForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: SubmitterNameForm,
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
            name: SubmitterNameForm::from(value.name),
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
        self.validate_postal_code(csrf_token, submitter_result)
    }

    pub fn validate_update_with_checks(
        self,
        current: &ListSubmitterData,
        csrf_token: &TokenValue,
    ) -> Result<ListSubmitterData, Box<FormData<Self>>> {
        let submitter_result = self.clone().validate_update(current, csrf_token);
        self.validate_postal_code(csrf_token, submitter_result)
    }

    fn validate_postal_code(
        self,
        csrf_token: &TokenValue,
        submitter_result: Result<ListSubmitterData, FormData<Self>>,
    ) -> Result<ListSubmitterData, Box<FormData<Self>>> {
        match CountryCode::from_str(&self.address.country) {
            Ok(country) if country.is_nl() => {
                let dutch_postalcode_result = PostalCode::from_str(&self.address.postal_code);
                if let Err(error) = dutch_postalcode_result {
                    let mut errors = vec![("address.postal_code".to_string(), error)];
                    if let Err(form_data) = submitter_result {
                        errors.extend(form_data.errors());
                    }
                    return Err(Box::new(FormData::new_with_errors(
                        self, csrf_token, errors,
                    )));
                }
                Ok(submitter_result?)
            }
            _ => Ok(submitter_result?),
        }
    }
    // TODO replace all International address form validates with the "with_check" variant
    // TODO unit test
    // TODO refactor personal_data_form in the same way
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{Address, CountryCode};

    #[test]
    fn validate_create_uses_dutch_address_when_country_is_empty() {
        let csrf_token = crate::form::generate_csrf_token();
        let form = ListSubmitterForm {
            name: SubmitterNameForm {
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

        let submitter: ListSubmitter = form.validate_create(&csrf_token).expect("submitter").into();

        assert!(matches!(submitter.address, Address::Dutch(_)));
    }

    #[test]
    fn validate_create_uses_international_address_when_country_is_foreign() {
        let csrf_token = crate::form::generate_csrf_token();
        let form = ListSubmitterForm {
            name: SubmitterNameForm {
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

        let submitter: ListSubmitter = form.validate_create(&csrf_token).expect("submitter").into();

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
}
