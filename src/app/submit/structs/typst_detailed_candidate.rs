use serde::Serialize;

use crate::{
    AppError,
    candidates::Candidate,
    common::{Address, BsnOrNoneConfirmed},
    core::ModelLocale,
    submit::structs::{
        typst_candidate::TypstCandidate, typst_person::TypstPerson,
        typst_postal_address::TypstPostalAddress,
    },
};

#[derive(Debug, Serialize)]
pub struct TypstDetailedCandidate {
    pub candidate: TypstCandidate,
    pub initials_no_gender: String,
    pub bsn: Option<String>,
    pub representative: Option<TypstPerson>,
    pub postal_address: Option<TypstPostalAddress>,
}

impl TypstDetailedCandidate {
    pub fn try_from(candidate: &Candidate, locale: ModelLocale) -> Result<Self, AppError> {
        let (representative, postal_address) = if candidate.person.lives_in_nl() {
            (
                None,
                Some(TypstPostalAddress::try_from(&Address::Dutch(
                    candidate.person.address.clone(),
                ))?),
            )
        } else {
            (
                Some(TypstPerson::try_from(&candidate.person.representative)?),
                None,
            )
        };
        let bsn = if candidate.person.personal_data.bsn == Some(BsnOrNoneConfirmed::NoneConfirmed) {
            None
        } else {
            Some(
                candidate
                    .person
                    .personal_data
                    .bsn
                    .as_ref()
                    .ok_or(AppError::IncompleteData("missing bsn"))?
                    .to_exposed_string(),
            )
        };

        Ok(Self {
            candidate: TypstCandidate::try_from(candidate, locale)?,
            initials_no_gender: candidate.person.name.initials_with_first_name(),
            bsn,
            representative,
            postal_address,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{
        candidate_lists::CandidateListId,
        common::{CountryCode, DutchAddress, HouseNumber, Locality, PostalCode, StreetName},
        persons::PersonId,
        test_utils::sample_person,
    };

    #[test]
    fn dutch_candidate_with_postal_address() {
        let mut candidate = Candidate {
            list_id: CandidateListId::new(),
            position: 1,
            person: sample_person(PersonId::new()),
        };
        candidate.person.personal_data.country = Some(CountryCode::from_str("NL").unwrap());
        let typst_candidate =
            TypstDetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap();

        assert_eq!(
            typst_candidate.postal_address.unwrap().postal_code,
            candidate.person.address.postal_code.unwrap().to_string()
        );
        assert!(typst_candidate.representative.is_none());
    }

    #[test]
    fn non_dutch_candidate_with_representative() {
        let mut candidate = Candidate {
            list_id: CandidateListId::new(),
            position: 1,
            person: sample_person(PersonId::new()),
        };
        candidate.person.personal_data.country = Some(CountryCode::from_str("BE").unwrap());
        candidate.person.representative.address = DutchAddress {
            street_name: Some(StreetName::from_str("street name").unwrap()),
            house_number: Some(HouseNumber::from_str("4").unwrap()),
            house_number_addition: None,
            postal_code: Some(PostalCode::from_str("1234AB").unwrap()),
            locality: Some(Locality::from_str("Amsterdam").unwrap()),
        };
        let typst_candidate =
            TypstDetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap();

        assert!(typst_candidate.postal_address.is_none());
        assert_eq!(
            typst_candidate.representative.unwrap().last_name,
            candidate.person.representative.name.last_name_with_prefix()
        );
    }

    #[test]
    fn dutch_candidate_without_postal_address() {
        let mut candidate = Candidate {
            list_id: CandidateListId::new(),
            position: 1,
            person: sample_person(PersonId::new()),
        };
        candidate.person.personal_data.country = Some(CountryCode::from_str("NL").unwrap());
        candidate.person.address.street_name = None;
        let err = TypstDetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap_err();
        assert!(matches!(err, AppError::IncompleteData(_)));
    }

    #[test]
    fn non_dutch_candidate_without_representative() {
        let mut candidate = Candidate {
            list_id: CandidateListId::new(),
            position: 1,
            person: sample_person(PersonId::new()),
        };
        candidate.person.personal_data.country = Some(CountryCode::from_str("BE").unwrap());
        let err = TypstDetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap_err();

        assert!(matches!(err, AppError::IncompleteData(_)));
    }

    #[test]
    fn dutch_candidate_without_bsn_confirmed() {
        let mut candidate = Candidate {
            list_id: CandidateListId::new(),
            position: 1,
            person: sample_person(PersonId::new()),
        };
        candidate.person.personal_data.bsn = Some(BsnOrNoneConfirmed::NoneConfirmed);

        let typst_candidate =
            TypstDetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap();

        assert_eq!(typst_candidate.bsn, None);
    }

    #[test]
    fn dutch_candidate_without_bsn() {
        let mut candidate = Candidate {
            list_id: CandidateListId::new(),
            position: 1,
            person: sample_person(PersonId::new()),
        };

        candidate.person.personal_data.bsn = None;

        let err = TypstDetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap_err();

        assert!(matches!(err, AppError::IncompleteData(_)))
    }
}
