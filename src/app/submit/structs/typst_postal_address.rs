use serde::Serialize;

use crate::{AppError, common::Address};

#[derive(Debug, Serialize)]
pub struct TypstPostalAddress {
    pub street_address: String,
    pub postal_code: String,
    pub locality: String,
}

impl TryFrom<&Address> for TypstPostalAddress {
    type Error = AppError;

    fn try_from(address: &Address) -> Result<Self, Self::Error> {
        Ok(TypstPostalAddress {
            street_address: address
                .address_line_1()
                .ok_or(AppError::IncompleteData("Missing street address"))?,
            postal_code: address
                .postal_code()
                .ok_or(AppError::IncompleteData("Missing postal code"))?,
            locality: address
                .locality()
                .ok_or(AppError::IncompleteData("Missing locality"))?,
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
        submitter.address = Address::Dutch(crate::common::DutchAddress {
            postal_code: None,
            ..submitter.address.as_dutch().unwrap().clone()
        });

        let err = TypstPostalAddress::try_from(&submitter.address).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing postal code")
        ));
    }

    #[test]
    fn typst_person_from_substitute_submitter_requires_address_line() {
        let mut submitter = sample_list_submitter(ListSubmitterId::new());
        submitter.address = Address::Dutch(crate::common::DutchAddress {
            street_name: None,
            ..submitter.address.as_dutch().unwrap().clone()
        });

        let err = TypstPostalAddress::try_from(&submitter.address).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing street address")
        ));
    }
}
