use crate::{ElectoralDistrict, Locale, trans};

use super::DateOfBirth;

/// Problem severities, in increasing order of severity
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Severity {
    Info,
    Warn,
    Error,
}

impl Severity {
    pub fn class(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warn => "warning",
            Severity::Error => "error",
        }
    }
}

pub trait Problematic {
    /// Returns all potential problems of its own and of all children
    fn get_problems(&self) -> Vec<PotentialProblems>;

    /// Returns true if there are no problems
    fn is_all_good(&self) -> bool {
        self.get_problems().is_empty()
    }

    /// Returns the highest severity of the problems, or None if there are no problems
    fn highest_severity(&self) -> Option<Severity> {
        self.get_problems().into_iter().map(|p| p.severity()).max()
    }

    /// Returns the CSS class associated with the highest severity
    fn highest_severity_class(&self) -> &'static str {
        match self.highest_severity() {
            None => "ok",
            Some(severity) => severity.class(),
        }
    }

    fn has_severity_or_higher(&self, severity: Severity) -> bool {
        self.get_problems().iter().any(|p| p.severity() >= severity)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum PotentialProblems {
    // candidate list
    NoCandidates,
    TooManyCandidates { actual: usize, max: usize },
    DuplicateDistricts { duplicates: Vec<ElectoralDistrict> },
    NoDistricts,
    FewCandidatesWithFirstName { count: usize, total: usize },
    FewCandidatesWithoutFirstName { count: usize, total: usize },
    FewCandidatesWithGender { count: usize, total: usize },
    FewCandidatesWithoutGender { count: usize, total: usize },

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
    VeryOldDateOfBirth,
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
            PotentialProblems::FewCandidatesWithFirstName { count, total } => {
                if *count == 1 {
                    trans!(
                        "problems.few_candidates_with_first_name_one",
                        *locale,
                        total
                    )
                } else {
                    trans!(
                        "problems.few_candidates_with_first_name",
                        *locale,
                        count,
                        total
                    )
                }
            }
            PotentialProblems::FewCandidatesWithoutFirstName { count, total } => {
                if *count == 1 {
                    trans!(
                        "problems.few_candidates_without_first_name_one",
                        *locale,
                        total
                    )
                } else {
                    trans!(
                        "problems.few_candidates_without_first_name",
                        *locale,
                        count,
                        total
                    )
                }
            }
            PotentialProblems::FewCandidatesWithGender { count, total } => {
                if *count == 1 {
                    trans!("problems.few_candidates_with_gender_one", *locale, total)
                } else {
                    trans!("problems.few_candidates_with_gender", *locale, count, total)
                }
            }
            PotentialProblems::FewCandidatesWithoutGender { count, total } => {
                if *count == 1 {
                    trans!("problems.few_candidates_without_gender_one", *locale, total)
                } else {
                    trans!(
                        "problems.few_candidates_without_gender",
                        *locale,
                        count,
                        total
                    )
                }
            }

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
            PotentialProblems::VeryOldDateOfBirth => {
                trans!(
                    "problems.very_old_date_of_birth",
                    *locale,
                    DateOfBirth::WARN_AGE
                )
            }
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
            PotentialProblems::FewCandidatesWithFirstName { .. } => Severity::Info,
            PotentialProblems::FewCandidatesWithoutFirstName { .. } => Severity::Info,
            PotentialProblems::FewCandidatesWithGender { .. } => Severity::Info,
            PotentialProblems::FewCandidatesWithoutGender { .. } => Severity::Info,

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
            PotentialProblems::VeryOldDateOfBirth => Severity::Warn,
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
