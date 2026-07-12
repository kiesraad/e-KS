use crate::{
    AppError, common::Address, list_submitters::ListSubmitter, models::inputs::Person,
    persons::Representative,
};

impl TryFrom<ListSubmitter> for Person {
    type Error = AppError;

    fn try_from(submitter: ListSubmitter) -> Result<Self, Self::Error> {
        Ok(Person {
            last_name: submitter.name.last_name_with_prefix(),
            initials: submitter.name.initials_with_first_name(),
            postal_address: (&submitter.address).into(),
        })
    }
}

impl From<&Representative> for Person {
    fn from(representative: &Representative) -> Self {
        Person {
            last_name: representative.name.last_name_with_prefix(),
            initials: representative.name.initials_with_first_name(),
            postal_address: (&Address::Dutch(representative.address.clone())).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{list_submitters::ListSubmitterId, test_utils::sample_list_submitter};

    #[test]
    fn person_from_list_submitter_maps_fields() -> Result<(), AppError> {
        let submitter = sample_list_submitter(ListSubmitterId::new());
        let person = Person::try_from(submitter)?;

        assert_eq!(person.last_name, "Bos");
        assert_eq!(person.initials, "E.F.");
        assert_eq!(person.postal_address.street_address, "Coolsingel 5B");
        assert_eq!(person.postal_address.postal_code, "3011CC".to_string());
        assert_eq!(person.postal_address.locality, "Rotterdam");

        Ok(())
    }
}
