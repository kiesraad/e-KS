use serde::Serialize;

use crate::{
    AppError,
    common::{DutchAddress, PostalCode},
};

#[derive(Debug, Serialize)]
pub struct TypstPostalAddress {
    pub street_address: String,
    pub postal_code: PostalCode,
    pub locality: String,
}

impl TryFrom<&DutchAddress> for TypstPostalAddress {
    type Error = AppError;

    fn try_from(address: &DutchAddress) -> Result<Self, Self::Error> {
        Ok(TypstPostalAddress {
            street_address: address
                .address_line_1()
                .ok_or(AppError::IncompleteData("Missing street address"))?,
            postal_code: address
                .postal_code
                .clone()
                .ok_or(AppError::IncompleteData("Missing postal code"))?,
            locality: address
                .locality
                .clone()
                .ok_or(AppError::IncompleteData("Missing locality"))?
                .to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{list_submitters::ListSubmitterId, test_utils::sample_list_submitter};

    #[test]
    fn typst_person_from_list_submitter_requires_postal_code() {
        let mut submitter = sample_list_submitter(ListSubmitterId::new());
        submitter.address.postal_code = None;

        let err = TypstPostalAddress::try_from(&submitter.address).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing postal code")
        ));
    }

    #[test]
    fn typst_person_from_substitute_submitter_requires_address_line() {
        let mut submitter = sample_list_submitter(ListSubmitterId::new());
        submitter.address.street_name = None;

        let err = TypstPostalAddress::try_from(&submitter.address).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing street address")
        ));
    }
}
