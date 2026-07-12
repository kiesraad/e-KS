use crate::{common::Address, models::inputs::PostalAddress};

impl From<&Address> for PostalAddress {
    fn from(address: &Address) -> Self {
        // Incomplete postal addresses cause warnings but not prevent export
        PostalAddress {
            street_address: address.address_line_1().unwrap_or_default(),
            postal_code: address.postal_code().unwrap_or_default(),
            locality: address.locality().unwrap_or_default(),
        }
    }
}
