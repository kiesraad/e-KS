use crate::{
    AppError,
    candidates::Candidate,
    common::{Address, BsnOrNoneConfirmed},
    core::ModelLocale,
    models::inputs::{Candidate as ModelCandidate, DetailedCandidate, Person, PostalAddress},
};

impl DetailedCandidate {
    pub fn try_from(candidate: &Candidate, locale: ModelLocale) -> Result<Self, AppError> {
        let (representative, postal_address) = if candidate.person.lives_in_nl() {
            (
                None,
                Some(PostalAddress::from(&Address::Dutch(
                    candidate.person.address.clone(),
                ))),
            )
        } else {
            (
                Some(Person::from(
                    candidate
                        .person
                        .representative
                        .as_ref()
                        .ok_or(AppError::IncompleteData("missing representative"))?,
                )),
                None,
            )
        };

        // BSNs cause warnings but don't prevent export
        let bsn = match candidate.person.personal_data.bsn.as_ref() {
            Some(BsnOrNoneConfirmed::Bsn(bsn)) => Some(bsn.to_exposed_string()),
            _ => None,
        };

        Ok(Self {
            candidate: ModelCandidate::try_from(candidate, locale)?,
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
        common::{
            BsnOrNoneConfirmed, CountryCode, DutchAddress, FullName, HouseNumber, Initials,
            LastName, Locality, PostalCode, StreetName,
        },
        persons::{PersonId, Representative},
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
        let detailed_candidate = DetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap();

        assert_eq!(
            detailed_candidate.postal_address.unwrap().postal_code,
            candidate.person.address.postal_code.unwrap().to_string()
        );
        assert!(detailed_candidate.representative.is_none());
    }

    #[test]
    fn non_dutch_candidate_with_representative() {
        let mut candidate = Candidate {
            list_id: CandidateListId::new(),
            position: 1,
            person: sample_person(PersonId::new()),
        };
        candidate.person.personal_data.country = Some(CountryCode::from_str("BE").unwrap());
        candidate.person.representative = Some(Representative {
            name: FullName {
                first_name: Some("Anne".parse().unwrap()),
                last_name: LastName::from_str("Dijk").unwrap(),
                last_name_prefix: None,
                initials: Initials::from_str("A.B.").unwrap(),
            },
            address: DutchAddress {
                street_name: Some(StreetName::from_str("street name").unwrap()),
                house_number: Some(HouseNumber::from_str("4").unwrap()),
                house_number_addition: None,
                postal_code: Some(PostalCode::from_str("1234AB").unwrap()),
                locality: Some(Locality::from_str("Amsterdam").unwrap()),
                known_in_bag: Some(true),
            },
        });
        let detailed_candidate = DetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap();

        assert!(detailed_candidate.postal_address.is_none());
        assert_eq!(
            detailed_candidate.representative.unwrap().last_name,
            candidate
                .person
                .representative
                .as_ref()
                .unwrap()
                .name
                .last_name_with_prefix()
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
        let detailed_candidate = DetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap();
        assert_eq!(
            detailed_candidate.postal_address.unwrap().street_address,
            ""
        );
    }

    #[test]
    fn non_dutch_candidate_without_representative() {
        let mut candidate = Candidate {
            list_id: CandidateListId::new(),
            position: 1,
            person: sample_person(PersonId::new()),
        };
        candidate.person.personal_data.country = Some(CountryCode::from_str("BE").unwrap());
        let err = DetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap_err();

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

        let detailed_candidate = DetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap();

        assert_eq!(detailed_candidate.bsn, None);
    }

    #[test]
    fn dutch_candidate_without_bsn() {
        let mut candidate = Candidate {
            list_id: CandidateListId::new(),
            position: 1,
            person: sample_person(PersonId::new()),
        };

        candidate.person.personal_data.bsn = None;

        let detailed_candidate = DetailedCandidate::try_from(&candidate, ModelLocale::Nl).unwrap();

        assert_eq!(detailed_candidate.bsn, None);
    }
}
