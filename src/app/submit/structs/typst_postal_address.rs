use serde::Serialize;

use crate::common::Address;

#[derive(Debug, Serialize)]
pub struct TypstPostalAddress {
    pub street_address: String,
    pub postal_code: String,
    pub locality: String,
}

impl From<&Address> for TypstPostalAddress {
    fn from(address: &Address) -> Self {
        // Incomplete postal addresses cause warnings but not prevent export
        TypstPostalAddress {
            street_address: address.address_line_1().unwrap_or_default(),
            postal_code: address.postal_code().unwrap_or_default(),
            locality: address.locality().unwrap_or_default(),
        }
    }
}
