use serde::Serialize;

use crate::{
    AppError, AppStore, candidate_lists::CandidateList, common::Address,
    list_submitters::ListSubmitter, persons::Representative,
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

pub fn substitute_submitter_from_ids(
    list: &CandidateList,
    store: AppStore,
) -> Result<Vec<TypstPerson>, AppError> {
    list.substitute_list_submitter_ids
        .iter()
        .map(|id| match store.get_substitute_submitter(*id) {
            Ok(submitter) => submitter.try_into(),
            Err(_) => Err(AppError::IntegrityViolation),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::LastName, list_submitters::ListSubmitterId, test_utils::sample_list_submitter,
    };

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

    #[tokio::test]
    async fn substitute_submitter_from_ids_resolves_submitters() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let submitter_a = sample_list_submitter(ListSubmitterId::new());
        let mut submitter_b = sample_list_submitter(ListSubmitterId::new());
        submitter_b.name.last_name = "Janssen".parse::<LastName>().expect("last name");

        submitter_a.create_substitute(&store).await?;
        submitter_b.create_substitute(&store).await?;

        let list = CandidateList {
            substitute_list_submitter_ids: vec![submitter_a.id, submitter_b.id],
            ..Default::default()
        };

        let resolved = substitute_submitter_from_ids(&list, store)?;

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].last_name, "Bos");
        assert_eq!(resolved[1].last_name, "Janssen");

        Ok(())
    }

    #[tokio::test]
    async fn substitute_submitter_from_ids_returns_integrity_error_on_missing() {
        let store = AppStore::new_for_test();
        let list = CandidateList {
            substitute_list_submitter_ids: vec![ListSubmitterId::new()],
            ..Default::default()
        };

        let err = substitute_submitter_from_ids(&list, store).unwrap_err();
        assert!(matches!(err, AppError::IntegrityViolation));
    }
}
