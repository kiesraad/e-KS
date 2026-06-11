use serde::Serialize;

use crate::{Locale, trans};

use super::DateOfBirth;

/// Problem severities, in increasing order of severity
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Serialize)]
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

#[derive(Debug, Clone)]
pub struct Problems {
    pub potential_problems: Vec<PotentialProblems>,
    pub info_problems: Vec<InfoProblems>,
}

impl Problems {
    pub fn new_empty() -> Self {
        Self {
            potential_problems: Vec::new(),
            info_problems: Vec::new(),
        }
    }

    /// Returns true if there are no problems and/or info problems
    pub fn is_all_good(&self) -> bool {
        self.potential_problems.is_empty() && self.info_problems.is_empty()
    }

    /// Returns the highest severity of the problems, or None if there are no problems
    pub fn highest_severity(&self) -> Option<Severity> {
        if !self.potential_problems.is_empty() {
            self.potential_problems.iter().map(|p| p.severity()).max()
        } else if !self.info_problems.is_empty() {
            Some(Severity::Info)
        } else {
            None
        }
    }

    /// Returns the CSS class associated with the highest severity
    pub fn highest_severity_class(&self) -> &'static str {
        self.highest_severity()
            .map(|severity| severity.class())
            .unwrap_or("ok")
    }

    pub fn has_severity_or_higher(&self, severity: Severity) -> bool {
        self.highest_severity()
            .map(|highest| highest >= severity)
            .unwrap_or(false)
    }

    /// Get a summary of the potential problems, if any
    pub fn problem_summary(&self, locale: &Locale) -> Option<String> {
        if self.potential_problems.is_empty() && self.info_problems.is_empty() {
            return None;
        }

        let potential_problems = self.potential_problems.iter().map(|p| p.translate(locale));
        let info_problems = self.info_problems.iter().map(|p| p.translate(locale));

        Some(
            potential_problems
                .chain(info_problems)
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }

    /// merge multiple Problems struct by concatenating all potential and info problems
    pub fn merge(problems: Vec<Self>) -> Self {
        let mut result = Self {
            potential_problems: Vec::new(),
            info_problems: Vec::new(),
        };
        for problem in problems {
            result.potential_problems.extend(problem.potential_problems);
            result.info_problems.extend(problem.info_problems);
        }
        result
    }
}

pub trait Problematic<T> {
    /// Returns all potential problems of its own and of all children
    fn get_problems(&self, additional_data: T) -> Problems;
}

#[derive(Debug, Clone)]
pub struct WithProblems<T> {
    pub data: T,
    pub problems: Problems,
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub enum PotentialProblems {
    // candidate list
    NoCandidateList,
    NoCandidates,
    TooManyCandidates {
        count: usize,
    },
    DuplicateDistricts,
    NoDistricts,

    // political group
    NoLegalName,
    NoDisplayName,
    NoAuthorisedAgent,
    NoListSubmitter,
    TooManyAuthorizedNames {
        count: usize,
    },
    TooFewAuthorizedNames {
        count: usize,
    },

    // representative wrapper
    RepresentativeProblem(Box<PotentialProblems>),

    // personal data
    NoBsn,
    NoPlaceOfResidence,
    UnknownPlaceOfResidence,
    NoCountryOfResidence,
    NoDateOfBirth,
    NoRepresentative,
    TooYoungDateOfBirth,

    // name related
    NoInitials(Severity),
    NoLastName(Severity),

    // address related
    UnknownAddress,
    IncompleteAddress {
        severity: Severity,
        problems: Vec<EmptyAddressProblems>,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub enum EmptyAddressProblems {
    StreetName,
    HouseNumber,
    PostalCode,
    Locality,
    Country,
}

impl PotentialProblems {
    pub fn translate(&self, locale: &Locale) -> String {
        match self {
            // candidate list
            PotentialProblems::NoCandidateList => trans!("problems.no_candidate_list", *locale),
            PotentialProblems::NoCandidates => trans!("problems.no_candidates", *locale),
            PotentialProblems::TooManyCandidates { count } => {
                if *count == 1 {
                    trans!("problems.too_many_candidates_one", *locale)
                } else {
                    trans!("problems.too_many_candidates", *locale, count)
                }
            }
            PotentialProblems::DuplicateDistricts => {
                trans!("problems.duplicate_districts", *locale)
            }
            PotentialProblems::NoDistricts => trans!("problems.no_districts", *locale),

            // political group
            PotentialProblems::NoLegalName => trans!("problems.no_legal_name", *locale),
            PotentialProblems::NoDisplayName => trans!("problems.no_display_name", *locale),
            PotentialProblems::NoListSubmitter => trans!("problems.no_list_submitter", *locale),
            PotentialProblems::NoAuthorisedAgent => trans!("problems.no_authorised_agent", *locale),
            PotentialProblems::TooManyAuthorizedNames { count } => {
                if *count == 1 {
                    trans!("problems.too_many_authorized_names_one", *locale)
                } else {
                    trans!("problems.too_many_authorized_names", *locale, count)
                }
            }
            PotentialProblems::TooFewAuthorizedNames { count } => {
                if *count == 1 {
                    trans!("problems.too_few_authorized_names_one", *locale)
                } else {
                    trans!("problems.too_few_authorized_names", *locale, count)
                }
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
            PotentialProblems::UnknownPlaceOfResidence => {
                trans!("problems.unknown_place_of_residence", *locale)
            }
            PotentialProblems::NoCountryOfResidence => {
                trans!("problems.no_country_of_residence", *locale)
            }
            PotentialProblems::NoDateOfBirth => trans!("problems.no_date_of_birth", *locale),
            PotentialProblems::NoRepresentative => trans!("problems.no_representative", *locale),
            PotentialProblems::TooYoungDateOfBirth => {
                trans!("problems.candidate_too_young", *locale,)
            }

            // name related
            PotentialProblems::NoInitials(..) => trans!("problems.no_initials", *locale),
            PotentialProblems::NoLastName(..) => trans!("problems.no_last_name", *locale),

            // address related
            PotentialProblems::UnknownAddress => trans!("problems.unknown_address", *locale),
            PotentialProblems::IncompleteAddress { .. } => {
                trans!("problems.incomplete_address", *locale)
            }
        }
    }

    pub fn severity(&self) -> Severity {
        match &self {
            // candidate list
            PotentialProblems::NoCandidateList => Severity::Error,
            PotentialProblems::NoCandidates => Severity::Error,
            PotentialProblems::TooManyCandidates { .. } => Severity::Warn,
            PotentialProblems::DuplicateDistricts => Severity::Error,
            PotentialProblems::NoDistricts => Severity::Error,

            // political group
            PotentialProblems::NoLegalName => Severity::Warn,
            PotentialProblems::NoDisplayName => Severity::Error,
            PotentialProblems::NoListSubmitter => Severity::Error,
            PotentialProblems::NoAuthorisedAgent => Severity::Warn,
            PotentialProblems::TooManyAuthorizedNames { .. } => Severity::Error,
            PotentialProblems::TooFewAuthorizedNames { .. } => Severity::Warn,

            // representative wrapper
            PotentialProblems::RepresentativeProblem(inner) => inner.severity(),

            // personal data
            PotentialProblems::NoBsn => Severity::Warn,
            PotentialProblems::TooYoungDateOfBirth => Severity::Warn,
            PotentialProblems::NoPlaceOfResidence => Severity::Error,
            PotentialProblems::UnknownPlaceOfResidence => Severity::Warn,
            PotentialProblems::NoCountryOfResidence => Severity::Error,
            PotentialProblems::NoDateOfBirth => Severity::Error,
            PotentialProblems::NoRepresentative => Severity::Warn,

            // name related
            PotentialProblems::NoInitials(severity) => *severity,
            PotentialProblems::NoLastName(severity) => *severity,

            // address related
            PotentialProblems::UnknownAddress => Severity::Warn,
            PotentialProblems::IncompleteAddress { severity, .. } => *severity,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub enum InfoProblems {
    FewCandidatesWithFirstName { count: usize, total: usize },
    FewCandidatesWithoutFirstName { count: usize, total: usize },
    FewCandidatesWithGender { count: usize, total: usize },
    FewCandidatesWithoutGender { count: usize, total: usize },
    NoPreviousElectionResults,
    NoSubstituteSubmitter,
    NoListDesignation,
    VeryOldDateOfBirth,
    NoInitials,
    NoLastName,
    IncompleteAddress { problems: Vec<EmptyAddressProblems> },
}

impl InfoProblems {
    pub fn translate(&self, locale: &Locale) -> String {
        match self {
            InfoProblems::NoInitials => trans!("problems.no_initials", *locale),
            InfoProblems::NoLastName => trans!("problems.no_last_name", *locale),
            InfoProblems::VeryOldDateOfBirth => {
                trans!(
                    "problems.very_old_date_of_birth",
                    *locale,
                    DateOfBirth::WARN_AGE
                )
            }
            InfoProblems::NoSubstituteSubmitter => {
                trans!("problems.no_substitute_submitter", *locale)
            }
            InfoProblems::NoListDesignation => trans!("problems.no_designation_type", *locale),
            InfoProblems::NoPreviousElectionResults => {
                trans!("problems.no_previous_election_results", *locale)
            }
            InfoProblems::FewCandidatesWithFirstName { count, total, .. } => {
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
            InfoProblems::FewCandidatesWithoutFirstName { count, total, .. } => {
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
            InfoProblems::FewCandidatesWithGender { count, total, .. } => {
                if *count == 1 {
                    trans!("problems.few_candidates_with_gender_one", *locale, total)
                } else {
                    trans!("problems.few_candidates_with_gender", *locale, count, total)
                }
            }
            InfoProblems::FewCandidatesWithoutGender { count, total, .. } => {
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
            InfoProblems::IncompleteAddress { .. } => {
                trans!("problems.incomplete_address", *locale)
            }
        }
    }

    pub fn severity(&self) -> Severity {
        Severity::Info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_order() {
        assert!(Severity::Info < Severity::Warn);
        assert!(Severity::Warn < Severity::Error);
    }

    #[test]
    fn highest_severity_none_when_no_problems() {
        let no_problems = Problems {
            potential_problems: Vec::new(),
            info_problems: Vec::new(),
        };
        assert_eq!(no_problems.highest_severity(), None);
        assert_eq!(no_problems.highest_severity_class(), "ok");
        assert!(!no_problems.has_severity_or_higher(Severity::Info,));
        assert!(!no_problems.has_severity_or_higher(Severity::Warn,));
        assert!(!no_problems.has_severity_or_higher(Severity::Error,));
        assert!(no_problems.is_all_good());
    }

    #[test]
    fn highest_severity_info_when_only_info() {
        let only_info = Problems {
            info_problems: vec![InfoProblems::NoLastName],
            potential_problems: Vec::new(),
        };
        assert_eq!(only_info.highest_severity(), Some(Severity::Info));
        assert_eq!(only_info.highest_severity_class(), "info");
        assert!(only_info.has_severity_or_higher(Severity::Info));
        assert!(!only_info.has_severity_or_higher(Severity::Warn));
        assert!(!only_info.has_severity_or_higher(Severity::Error));
        assert!(!only_info.is_all_good());
    }

    #[test]
    fn highest_severity_warn_when_only_warnings() {
        let info_warn = Problems {
            info_problems: vec![InfoProblems::IncompleteAddress {
                problems: Vec::new(),
            }],
            potential_problems: vec![PotentialProblems::IncompleteAddress {
                severity: Severity::Warn,
                problems: Vec::new(),
            }],
        };
        assert_eq!(info_warn.highest_severity(), Some(Severity::Warn));
        assert_eq!(info_warn.highest_severity_class(), "warning");
        assert!(info_warn.has_severity_or_higher(Severity::Info,));
        assert!(info_warn.has_severity_or_higher(Severity::Warn,));
        assert!(!info_warn.has_severity_or_higher(Severity::Error,));
        assert!(!info_warn.is_all_good());
    }

    #[test]
    fn highest_severity_error_when_mix_of_severities() {
        let with_error = Problems {
            info_problems: vec![InfoProblems::IncompleteAddress {
                problems: Vec::new(),
            }],
            potential_problems: vec![
                PotentialProblems::IncompleteAddress {
                    severity: Severity::Warn,
                    problems: Vec::new(),
                },
                PotentialProblems::IncompleteAddress {
                    severity: Severity::Error,
                    problems: Vec::new(),
                },
            ],
        };
        assert_eq!(with_error.highest_severity(), Some(Severity::Error));
        assert_eq!(with_error.highest_severity_class(), "error");
        assert!(with_error.has_severity_or_higher(Severity::Info,));
        assert!(with_error.has_severity_or_higher(Severity::Warn,));
        assert!(with_error.has_severity_or_higher(Severity::Error,));
        assert!(!with_error.is_all_good());
    }

    #[test]
    fn no_problem_summary() {
        let no_problems = Problems::new_empty();
        assert_eq!(no_problems.problem_summary(&Locale::Nl), None);
    }

    #[test]
    fn single_problem_summary() {
        let problem = PotentialProblems::NoDistricts;
        let single_problems = Problems {
            potential_problems: vec![problem.clone()],
            info_problems: Vec::new(),
        };
        assert_eq!(
            single_problems.problem_summary(&Locale::Nl).unwrap(),
            problem.translate(&Locale::Nl)
        );
    }

    #[test]
    fn multiple_problem_summary() {
        let problems = [
            PotentialProblems::NoBsn,
            PotentialProblems::NoPlaceOfResidence,
            PotentialProblems::NoDateOfBirth,
            PotentialProblems::RepresentativeProblem(Box::new(
                PotentialProblems::IncompleteAddress {
                    severity: Severity::Warn,
                    problems: vec![
                        EmptyAddressProblems::StreetName,
                        EmptyAddressProblems::PostalCode,
                        EmptyAddressProblems::Locality,
                        EmptyAddressProblems::HouseNumber,
                        EmptyAddressProblems::Country,
                    ],
                },
            )),
        ];
        let info_problems = [InfoProblems::NoLastName, InfoProblems::NoListDesignation];
        let multiple_problems = Problems {
            potential_problems: problems.to_vec(),
            info_problems: info_problems.to_vec(),
        };
        let summary = multiple_problems.problem_summary(&Locale::Nl).unwrap();
        for problem in problems {
            assert!(summary.contains(&problem.translate(&Locale::Nl)));
        }
    }

    #[test]
    fn deviation_shows_numbers() {
        let info_problems = vec![
            InfoProblems::FewCandidatesWithFirstName {
                count: 2,
                total: 37,
            },
            InfoProblems::FewCandidatesWithGender {
                count: 2,
                total: 37,
            },
            InfoProblems::FewCandidatesWithoutFirstName {
                count: 2,
                total: 37,
            },
            InfoProblems::FewCandidatesWithoutGender {
                count: 2,
                total: 37,
            },
        ];

        for problem in info_problems {
            let summary = Problems {
                info_problems: vec![problem],
                potential_problems: Vec::new(),
            }
            .problem_summary(&Locale::Nl)
            .unwrap();
            assert!(summary.contains("2"));
            assert!(summary.contains("37"));
        }
    }
}
