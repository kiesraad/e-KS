use crate::{
    TokenValue,
    common::{FullNameForm, InternationalAddressForm},
    list_submitters::{ListSubmitter, ListSubmitterData},
};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "ListSubmitterData")]
#[serde(default)]
pub struct ListSubmitterForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: FullNameForm,
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
            name: FullNameForm::from(value.name),
            address: InternationalAddressForm::from(value.address),
            csrf_token: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{Address, CountryCode};

    #[test]
    fn validate_create_uses_dutch_address_when_country_is_empty() {
        let csrf_tokens = crate::CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let form = ListSubmitterForm {
            name: FullNameForm {
                first_name: String::new(),
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
            csrf_token,
        };

        let submitter: ListSubmitter = form
            .validate_create(&csrf_tokens)
            .expect("submitter")
            .into();

        assert!(matches!(submitter.address, Address::Dutch(_)));
    }

    #[test]
    fn validate_create_uses_international_address_when_country_is_foreign() {
        let csrf_tokens = crate::CsrfTokens::default();
        let csrf_token = csrf_tokens.issue().value;
        let form = ListSubmitterForm {
            name: FullNameForm {
                first_name: String::new(),
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
            csrf_token,
        };

        let submitter: ListSubmitter = form
            .validate_create(&csrf_tokens)
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
}
