use serde::Serialize;

use crate::{
    AppError, common::Address, list_submitters::ListSubmitter, persons::Representative,
    submit::structs::typst_postal_address::TypstPostalAddress,
};

#[derive(Debug, Serialize)]
pub struct TypstPerson {
    pub last_name: String,
    /// Initials as printed on the model, e.g., optionally including the first name
    pub initials: String,
    pub postal_address: TypstPostalAddress,
}

impl TryFrom<ListSubmitter> for TypstPerson {
    type Error = AppError;

    fn try_from(submitter: ListSubmitter) -> Result<Self, Self::Error> {
        Ok(TypstPerson {
            last_name: submitter.name.last_name_with_prefix(),
            initials: submitter.name.initials_with_first_name(),
            postal_address: (&submitter.address).try_into()?,
        })
    }
}

impl TryFrom<&Representative> for TypstPerson {
    type Error = AppError;

    fn try_from(representative: &Representative) -> Result<Self, Self::Error> {
        Ok(TypstPerson {
            last_name: representative.name.last_name_with_prefix(),
            initials: representative.name.initials_with_first_name(),
            postal_address: (&Address::Dutch(representative.address.clone())).try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{list_submitters::ListSubmitterId, test_utils::sample_list_submitter};

    #[test]
    fn typst_person_from_list_submitter_maps_fields() -> Result<(), AppError> {
        let submitter = sample_list_submitter(ListSubmitterId::new());
        let person = TypstPerson::try_from(submitter)?;

        assert_eq!(person.last_name, "Bos");
        assert_eq!(person.initials, "E.F.");
        assert_eq!(person.postal_address.street_address, "Coolsingel 5B");
        assert_eq!(person.postal_address.postal_code, "3011CC".to_string());
        assert_eq!(person.postal_address.locality, "Rotterdam");

        Ok(())
    }
}
