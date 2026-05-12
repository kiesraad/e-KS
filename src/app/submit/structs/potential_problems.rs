use axum_extra::routing::TypedPath as _;

use crate::{
    AppStore, ElectoralDistrict, Locale, QueryParamState,
    authorised_agents::AuthorisedAgent,
    candidate_lists::{CandidateList, CandidateListSummary},
    list_submitters::ListSubmitter,
    persons::Person,
    political_groups::PoliticalGroup,
    trans,
};

/// Aggregation struct for everything that can be missing or incomplete for a list submission
#[derive(Debug)]
pub struct Problems {
    pub general: GeneralProblems,
    pub candidates: Vec<PersonProblems<Person>>,
    pub lists: Vec<ListProblems>,
}

impl Problems {
    pub fn find_all(store: &AppStore) -> Self {
        let candidate_lists = CandidateListSummary::list(store);

        Self {
            general: Self::find_general_problems(store),
            candidates: {
                let mut seen = std::collections::HashSet::new();
                candidate_lists
                    .iter()
                    .flat_map(|list| list.list.candidates.iter())
                    .filter(|id| seen.insert(*id))
                    .filter_map(|id| store.get_person(*id).ok())
                    .filter_map(|person| {
                        let problems = person.get_problems();
                        (!problems.is_empty()).then(|| PersonProblems { person, problems })
                    })
                    .collect()
            },
            lists: candidate_lists
                .iter()
                .filter_map(|candidate_list| {
                    let problems = candidate_list.get_problems();
                    (!problems.is_empty()).then(|| ListProblems {
                        list: candidate_list.list.clone(),
                        problems,
                    })
                })
                .collect(),
        }
    }

    fn find_general_problems(store: &AppStore) -> GeneralProblems {
        let mut general = store.get_political_group().get_problems();

        let authorised_agents = store.get_authorised_agents();
        if authorised_agents.is_empty() {
            general.push(PotentialProblems::NoAuthorisedAgent);
        }
        let authorised_agents = authorised_agents
            .into_iter()
            .map(PersonProblems::new)
            .collect();

        let list_submitter = store.get_list_submitter();
        if list_submitter.is_empty() {
            general.push(PotentialProblems::NoListSubmitter);
        }
        let list_submitter = list_submitter.get_problems();

        let substitute_submitters = store.get_substitute_submitters();
        if substitute_submitters.is_empty() {
            general.push(PotentialProblems::NoSubstituteSubmitter);
        }
        let substitute_submitters = substitute_submitters
            .into_iter()
            .map(PersonProblems::new)
            .collect();

        GeneralProblems {
            general,
            authorised_agents,
            list_submitter,
            substitute_submitters,
        }
    }

    pub fn models_downloadable(&self) -> bool {
        let candidate_iter = self.candidates.iter().flat_map(|ci| &ci.problems);
        let list_iter = self.lists.iter().flat_map(|ci| &ci.problems);
        let general_iter = self.general.flatten();

        !candidate_iter
            .chain(list_iter)
            .chain(general_iter)
            .any(|ii| ii.severity() == Severity::Error)
    }
}

#[derive(Debug)]
pub struct GeneralProblems {
    pub general: Vec<PotentialProblems>,
    pub authorised_agents: Vec<PersonProblems<AuthorisedAgent>>,
    pub list_submitter: Vec<PotentialProblems>,
    pub substitute_submitters: Vec<PersonProblems<ListSubmitter>>,
}

impl GeneralProblems {
    pub fn flatten(&self) -> Vec<&PotentialProblems> {
        let mut result = Vec::new();

        result.extend(&self.general);
        result.extend(self.authorised_agents.iter().flat_map(|aa| &aa.problems));
        result.extend(
            self.substitute_submitters
                .iter()
                .flat_map(|ss| &ss.problems),
        );
        result.extend(&self.list_submitter);

        result
    }
}
#[derive(Debug)]
pub struct PersonProblems<T> {
    pub person: T,
    pub problems: Vec<PotentialProblems>,
}

impl<T: Problematic> PersonProblems<T> {
    fn new(person: T) -> Self {
        let problems = person.get_problems();
        PersonProblems { person, problems }
    }
}

#[derive(Debug)]
pub struct ListProblems {
    pub list: CandidateList,
    pub problems: Vec<PotentialProblems>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PotentialProblems {
    // candidate list
    NoCandidates,
    TooManyCandidates { actual: usize, max: usize },
    DuplicateDistricts { duplicates: Vec<ElectoralDistrict> },
    NoDistricts,

    // political group
    NoLegalName,
    NoDisplayName,
    NoPreviousElectionResults,
    NoAuthorisedAgent,
    NoListSubmitter,
    NoSubstituteSubmitter,

    // representative wrapper
    RepresentativeProblem(Box<PotentialProblems>),

    // personal data
    NoBsn,
    NoPlaceOfResidence,
    NoCountryOfResidence,
    NoDateOfBirth,
    NoRepresentative,

    // name related
    NoInitials(Severity),
    NoLastName(Severity),

    // address related
    NoStreetName(Severity),
    NoHouseNumber(Severity),
    NoPostalCode(Severity),
    NoLocality(Severity),
    NoCountry(Severity),
}

impl PotentialProblems {
    pub fn translate(&self, locale: &Locale) -> String {
        match self {
            // candidate list
            PotentialProblems::NoCandidates => trans!("problems.no_candidates", *locale),
            PotentialProblems::TooManyCandidates { actual, max } => {
                trans!("problems.too_many_candidates", *locale, actual, max)
            }
            PotentialProblems::DuplicateDistricts { duplicates } => {
                let districts = duplicates
                    .iter()
                    .map(|d| d.title((*locale).into()))
                    .collect::<Vec<_>>()
                    .join(", ");
                trans!("problems.duplicate_districts", *locale, districts)
            }
            PotentialProblems::NoDistricts => trans!("problems.no_districts", *locale),

            // political group
            PotentialProblems::NoLegalName => trans!("problems.no_legal_name", *locale),
            PotentialProblems::NoDisplayName => trans!("problems.no_display_name", *locale),
            PotentialProblems::NoPreviousElectionResults => {
                trans!("problems.no_previous_election_results", *locale)
            }
            PotentialProblems::NoListSubmitter => trans!("problems.no_list_submitter", *locale),
            PotentialProblems::NoAuthorisedAgent => trans!("problems.no_authorised_agent", *locale),
            PotentialProblems::NoSubstituteSubmitter => {
                trans!("problems.no_substitute_submitter", *locale)
            }

            // representative wrapper
            PotentialProblems::RepresentativeProblem(inner) => {
                let label = trans!("problems.representative", *locale);
                let problem = inner.translate(locale);
                format!("{label}: {problem}")
            }

            // personal data
            PotentialProblems::NoBsn => trans!("problems.no_bsn", *locale),
            PotentialProblems::NoPlaceOfResidence => {
                trans!("problems.no_place_of_residence", *locale)
            }
            PotentialProblems::NoCountryOfResidence => {
                trans!("problems.no_country_of_residence", *locale)
            }
            PotentialProblems::NoDateOfBirth => trans!("problems.no_date_of_birth", *locale),
            PotentialProblems::NoRepresentative => trans!("problems.no_representative", *locale),

            // name related
            PotentialProblems::NoInitials(_) => trans!("problems.no_initials", *locale),
            PotentialProblems::NoLastName(_) => trans!("problems.no_last_name", *locale),

            // address related
            PotentialProblems::NoStreetName(_) => trans!("problems.no_street_name", *locale),
            PotentialProblems::NoHouseNumber(_) => trans!("problems.no_house_number", *locale),
            PotentialProblems::NoPostalCode(_) => trans!("problems.no_postal_code", *locale),
            PotentialProblems::NoLocality(_) => trans!("problems.no_locality", *locale),
            PotentialProblems::NoCountry(_) => trans!("problems.no_country", *locale),
        }
    }

    pub fn candidate_list_fix_path(&self, list: &CandidateList) -> String {
        match self {
            PotentialProblems::NoCandidates => list.view_path().to_string(),
            PotentialProblems::TooManyCandidates { actual, max } => {
                let overflow = actual.saturating_sub(*max);
                list.view_path()
                    .with_query_params(QueryParamState::highlight_last(overflow))
                    .to_string()
            }
            _ => list.update_path().to_string(),
        }
    }

    pub fn person_fix_path(&self, person: &Person) -> String {
        match self {
            PotentialProblems::NoStreetName(_)
            | PotentialProblems::NoHouseNumber(_)
            | PotentialProblems::NoPostalCode(_)
            | PotentialProblems::NoLocality(_)
            | PotentialProblems::NoCountry(_) => person.update_address_path().to_string(),
            PotentialProblems::NoRepresentative | PotentialProblems::RepresentativeProblem(_) => {
                person.update_representative_path().to_string()
            }
            _ => person.update_path().to_string(),
        }
    }

    pub fn general_fix_path(&self) -> String {
        match self {
            PotentialProblems::NoAuthorisedAgent => AuthorisedAgent::list_path().to_string(),
            PotentialProblems::NoListSubmitter => ListSubmitter::update_path().to_string(),
            PotentialProblems::NoSubstituteSubmitter => ListSubmitter::view_path().to_string(),
            _ => PoliticalGroup::update_path().to_string(),
        }
    }

    pub fn severity_class(&self) -> &'static str {
        match self.severity() {
            Severity::Info => "info",
            Severity::Warn => "warning",
            Severity::Error => "error",
        }
    }

    pub fn severity(&self) -> Severity {
        match &self {
            // candidate list
            PotentialProblems::NoCandidates => Severity::Error,
            PotentialProblems::TooManyCandidates { .. } => Severity::Warn,
            PotentialProblems::DuplicateDistricts { .. } => Severity::Error,
            PotentialProblems::NoDistricts => Severity::Error,

            // political group
            PotentialProblems::NoLegalName => Severity::Warn,
            PotentialProblems::NoDisplayName => Severity::Error,
            PotentialProblems::NoPreviousElectionResults => Severity::Info,
            PotentialProblems::NoListSubmitter => Severity::Error,
            PotentialProblems::NoAuthorisedAgent => Severity::Warn,
            PotentialProblems::NoSubstituteSubmitter => Severity::Info,

            // representative wrapper
            PotentialProblems::RepresentativeProblem(inner) => inner.severity(),

            // personal data
            PotentialProblems::NoBsn => Severity::Warn,
            PotentialProblems::NoPlaceOfResidence => Severity::Error,
            PotentialProblems::NoCountryOfResidence => Severity::Error,
            PotentialProblems::NoDateOfBirth => Severity::Error,
            PotentialProblems::NoRepresentative => Severity::Warn,

            // name related
            PotentialProblems::NoInitials(severity) => *severity,
            PotentialProblems::NoLastName(severity) => *severity,

            // address related
            PotentialProblems::NoStreetName(severity) => *severity,
            PotentialProblems::NoHouseNumber(severity) => *severity,
            PotentialProblems::NoPostalCode(severity) => *severity,
            PotentialProblems::NoLocality(severity) => *severity,
            PotentialProblems::NoCountry(severity) => *severity,
        }
    }
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum Severity {
    Info,
    Warn,
    Error,
}

pub trait Problematic {
    /// returns all potential problems of its own and of all children
    fn get_problems(&self) -> Vec<PotentialProblems>;

    fn is_all_good(&self) -> bool {
        self.get_problems().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{sample_candidate_list, sample_person},
    };

    use super::*;

    fn empty_general() -> GeneralProblems {
        GeneralProblems {
            general: Vec::new(),
            authorised_agents: Vec::new(),
            list_submitter: Vec::new(),
            substitute_submitters: Vec::new(),
        }
    }

    #[test]
    fn is_printable() {
        assert!(
            Problems {
                general: empty_general(),
                candidates: Vec::new(),
                lists: Vec::new(),
            }
            .models_downloadable()
        );

        assert!(
            Problems {
                general: empty_general(),
                candidates: vec![],
                lists: vec![ListProblems {
                    list: sample_candidate_list(CandidateListId::new()),
                    problems: vec![PotentialProblems::TooManyCandidates {
                        actual: 12,
                        max: 12
                    }],
                }],
            }
            .models_downloadable()
        );

        assert!(
            !Problems {
                general: empty_general(),
                candidates: vec![PersonProblems {
                    person: sample_person(PersonId::new()),
                    problems: vec![PotentialProblems::NoCandidates]
                }],
                lists: Vec::new(),
            }
            .models_downloadable()
        );
    }
}
