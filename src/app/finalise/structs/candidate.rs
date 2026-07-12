use chrono::Datelike;
use tracing::error;

use crate::{
    AppError,
    candidates::Candidate,
    core::ModelLocale,
    models::inputs::{Candidate as ModelCandidate, Date},
};

impl From<crate::common::DateOfBirth> for Date {
    fn from(date: crate::common::DateOfBirth) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }
}

impl ModelCandidate {
    pub fn try_from(candidate: &Candidate, locale: ModelLocale) -> Result<Self, AppError> {
        Ok(Self {
            last_name: candidate.person.name.last_name_with_prefix(),
            initials: candidate.person.initials_as_printed_on_list(locale.into()),
            date_of_birth: candidate
                .person
                .personal_data
                .date_of_birth
                .clone()
                .ok_or(AppError::IncompleteData("Missing birth date for candidate"))?
                .into(),
            locality: candidate
                .person
                .personal_data
                .locality()
                .clone()
                .ok_or(AppError::IncompleteData("Missing locality for candidate"))?
                .to_string(),
            position: candidate.position,
        })
    }
}

pub fn ordered_candidates(
    candidates: &mut [crate::candidates::Candidate],
    locale: ModelLocale,
) -> Result<Vec<ModelCandidate>, AppError> {
    candidates.sort_by_key(|c| c.position);

    for (i, candidate) in candidates.iter().enumerate() {
        if candidate.position != i + 1 {
            error!(
                expected_position = i + 1,
                actual_position = candidate.position,
                person_id = %candidate.person.id,
                "Found a hole in candidate list",
            );
            return Err(AppError::IntegrityViolation);
        }
    }

    candidates
        .iter()
        .map(|c| ModelCandidate::try_from(c, locale))
        .collect::<Result<Vec<_>, _>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{sample_person, sample_person_with_last_name},
    };

    #[test]
    fn ordered_candidates_sorts_and_maps_people() -> Result<(), AppError> {
        let list_id = CandidateListId::new();
        let person_a = sample_person_with_last_name(PersonId::new(), "Alpha");
        let person_b = sample_person_with_last_name(PersonId::new(), "Beta");

        let mut candidates = vec![
            Candidate {
                list_id,
                position: 2,
                person: person_a,
            },
            Candidate {
                list_id,
                position: 1,
                person: person_b,
            },
        ];

        let ordered = ordered_candidates(&mut candidates, ModelLocale::Nl)?;

        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0].last_name, "Beta");
        assert_eq!(ordered[1].last_name, "Alpha");
        assert_eq!(ordered[0].date_of_birth.year, 1990);
        assert_eq!(ordered[0].date_of_birth.month, 2);
        assert_eq!(ordered[0].date_of_birth.day, 1);
        assert_eq!(ordered[0].locality, "Juinen");

        Ok(())
    }

    #[test]
    fn ordered_candidates_returns_error_on_hole() {
        let list_id = CandidateListId::new();
        let mut candidates = vec![
            Candidate {
                list_id,
                position: 1,
                person: sample_person(PersonId::new()),
            },
            Candidate {
                list_id,
                position: 3,
                person: sample_person(PersonId::new()),
            },
        ];

        let err = ordered_candidates(&mut candidates, ModelLocale::Nl).unwrap_err();
        assert!(matches!(err, AppError::IntegrityViolation));
    }

    #[test]
    fn candidate_requires_birth_date() {
        let list_id = CandidateListId::new();
        let mut candidate = Candidate {
            list_id,
            position: 18,
            person: sample_person(PersonId::new()),
        };
        candidate.person.personal_data.date_of_birth = None;

        let err = ModelCandidate::try_from(&candidate, ModelLocale::Nl).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing birth date for candidate")
        ));
    }

    #[test]
    fn candidate_requires_locality() {
        let list_id = CandidateListId::new();
        let mut candidate = Candidate {
            list_id,
            position: 18,
            person: sample_person(PersonId::new()),
        };
        candidate.person.personal_data.place_of_residence = None;

        let err = ModelCandidate::try_from(&candidate, ModelLocale::Nl).unwrap_err();
        assert!(matches!(
            err,
            AppError::IncompleteData("Missing locality for candidate")
        ));
    }

    #[test]
    fn date_from_common_date_copies_components() {
        let input: crate::common::DateOfBirth = "15-03-2001".parse().expect("date");
        let date = Date::from(input);

        assert_eq!(date.year, 2001);
        assert_eq!(date.month, 3);
        assert_eq!(date.day, 15);
    }
}
