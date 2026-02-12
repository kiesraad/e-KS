use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::OptionStringExt;

use super::{HouseNumber, HouseNumberAddition, Locality, PostalCode, StreetName};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DutchAddress {
    pub street_name: Option<StreetName>,
    pub house_number: Option<HouseNumber>,
    pub house_number_addition: Option<HouseNumberAddition>,
    pub locality: Option<Locality>,
    pub postal_code: Option<PostalCode>,
}

impl DutchAddress {
    pub fn is_complete(&self) -> bool {
        self.street_name.is_some()
            && self.house_number.is_some()
            && self.postal_code.is_some()
            && self.locality.is_some()
    }

    pub fn is_empty(&self) -> bool {
        self.street_name.is_empty_or_none()
            && self.house_number.is_empty_or_none()
            && self.house_number_addition.is_empty_or_none()
            && self.postal_code.is_empty_or_none()
            && self.locality.is_empty_or_none()
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "DutchAddress", timestamps = false)]
#[serde(default)]
pub struct DutchAddressForm {
    #[validate(parse = "Locality", optional)]
    pub locality: String,
    #[validate(parse = "PostalCode", optional)]
    pub postal_code: String,
    #[validate(parse = "HouseNumber", optional)]
    pub house_number: String,
    #[validate(parse = "HouseNumberAddition", optional)]
    pub house_number_addition: String,
    #[validate(parse = "StreetName", optional)]
    pub street_name: String,
}

impl From<DutchAddress> for DutchAddressForm {
    fn from(address: DutchAddress) -> Self {
        DutchAddressForm {
            locality: address.locality.to_string_or_default(),
            postal_code: address.postal_code.to_string_or_default(),
            house_number: address.house_number.to_string_or_default(),
            house_number_addition: address.house_number_addition.to_string_or_default(),
            street_name: address.street_name.to_string_or_default(),
        }
    }
}
