use crate::{
    AppError, ElectionConfig, Locale, Store,
    candidate_lists::{CandidateListId, FullCandidateList},
    common::PostalCode,
    core::ElectionType,
    persons::Person,
};
use chrono::{NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct H1 {
    election_name: String,
    election_type: ElectionType,
    electoral_districts: Vec<String>,
    designation: String,
    candidates: Vec<Candidate>,
    previously_seated: bool,
    substitute_submitter: Vec<SubstituteSubmitter>,
    timestamp: chrono::DateTime<Utc>,
}

impl H1 {
    pub fn new(
        store: &Store,
        list_id: CandidateListId,
        election: ElectionConfig,
    ) -> Result<Self, AppError> {
        let FullCandidateList {
            list,
            mut candidates,
        } = FullCandidateList::get(store, list_id)?;
        let substitute_submitters = store.get_substitute_submitters()?;

        Ok(Self {
            election_name: election.title().to_string(),
            election_type: election.election_type(),
            electoral_districts: list
                .electoral_districts
                .iter()
                .map(|d| d.title().to_string())
                .collect(),
            designation: store
                .get_political_group()?
                .display_name
                .ok_or(AppError::IntegrityViolation)?
                .to_string(),
            candidates: ordered_candidates(&mut candidates),
            // TODO
            previously_seated: true,
            substitute_submitter: list
                .substitute_list_submitter_ids
                .iter()
                .filter_map(|id| substitute_submitters.iter().find(|sub| sub.id == *id))
                .map(SubstituteSubmitter::from)
                .collect(),
            timestamp: Default::default(),
        })
    }
}

#[derive(Debug, Serialize)]
struct Candidate {
    last_name: String,
    /// Initials as printed on the model, e.g., optionally including the gender and first name
    initials: String,
    date_of_birth: NaiveDate,
    locality: String,
}

impl From<&Person> for Candidate {
    fn from(person: &Person) -> Self {
        Self {
            last_name: person.display_name(),
            // TODO locale
            initials: person.initials_as_printed_on_list(Locale::Nl),
            // FIXME expect
            date_of_birth: person.date_of_birth.expect("Must be complete").into(),
            // FIXME expect
            locality: person
                .place_of_residence
                .clone()
                .expect("Must be complete")
                .to_string(),
        }
    }
}

fn ordered_candidates(candidates: &mut [crate::candidates::Candidate]) -> Vec<Candidate> {
    candidates.sort_by(|a, b| a.position.cmp(&b.position));
    for (i, candidate) in candidates.iter().enumerate() {
        // FIXME proper error handling
        assert_eq!(i + 1, candidate.position);
    }

    candidates
        .iter()
        .map(|c| Candidate::from(&c.person))
        .collect()
}

#[derive(Debug, Serialize)]
struct SubstituteSubmitter {
    last_name: String,
    /// Initials as printed on the model, e.g., optionally including the gender and first name
    initials: String,
    postal_address: String,
    postal_code: PostalCode,
    locality: String,
}

impl From<&crate::substitute_list_submitters::SubstituteSubmitter> for SubstituteSubmitter {
    fn from(submitter: &crate::substitute_list_submitters::SubstituteSubmitter) -> Self {
        // TODO unwraps
        Self {
            last_name: submitter.name.last_name.to_string(),
            initials: submitter.name.initials.to_string(),
            postal_address: submitter.address.address_line_1().unwrap(),
            postal_code: submitter.address.postal_code.clone().unwrap(),
            locality: submitter.address.locality.clone().unwrap().to_string(),
        }
    }
}
