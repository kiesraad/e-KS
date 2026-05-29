use crate::{Locale, trans};

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

    /// Get a summary of the potential problems, if any
    fn problem_summary(&self, locale: &Locale) -> Option<String> {
        let problems = self.get_problems();

        if problems.is_empty() {
            return None;
        }

        Some(
            problems
                .iter()
                .map(|p| p.translate(locale))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum PotentialProblems {
    // candidate list
    NoCandidates,
    TooManyCandidates {
        actual: usize,
        max: usize,
    },
    DuplicateDistricts,
    NoDistricts,
    FewCandidatesWithFirstName {
        count: usize,
        total: usize,
    },
    FewCandidatesWithoutFirstName {
        count: usize,
        total: usize,
    },
    FewCandidatesWithGender {
        count: usize,
        total: usize,
    },
    FewCandidatesWithoutGender {
        count: usize,
        total: usize,
    },

    // political group
    NoLegalName,
    NoDisplayName,
    NoPreviousElectionResults,
    NoAuthorisedAgent,
    NoListSubmitter,
    NoSubstituteSubmitter,
    NoDesignationType,
    TooManyAuthorizedNames {
        actual: usize,
        max: usize,
    },
    TooFewAuthorizedNames {
        actual: usize,
        min: usize,
    },

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
    IncompleteAddress {
        severity: Severity,
        problems: Vec<EmptyAddressProblems>,
    },
}

#[derive(Clone, PartialEq, Debug)]
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
            PotentialProblems::NoCandidates => trans!("problems.no_candidates", *locale),
            PotentialProblems::TooManyCandidates { actual, max } => {
                trans!("problems.too_many_candidates", *locale, actual, max)
            }
            PotentialProblems::DuplicateDistricts => {
                trans!("problems.duplicate_districts", *locale)
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
            PotentialProblems::NoDesignationType => trans!("problems.no_legal_name", *locale),
            PotentialProblems::TooManyAuthorizedNames { actual, max } => {
                trans!("problems.too_many_authorized_names", *locale, actual, max)
            }
            PotentialProblems::TooFewAuthorizedNames { actual, min } => {
                trans!("problems.too_few_authorized_names", *locale, actual, min)
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
            PotentialProblems::IncompleteAddress { .. } => {
                trans!("problems.incomplete_address", *locale)
            }
        }
    }

    pub fn severity(&self) -> Severity {
        match &self {
            // candidate list
            PotentialProblems::NoCandidates => Severity::Error,
            PotentialProblems::TooManyCandidates { .. } => Severity::Warn,
            PotentialProblems::DuplicateDistricts => Severity::Error,
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
            PotentialProblems::NoDesignationType => Severity::Info,
            PotentialProblems::TooManyAuthorizedNames { .. } => Severity::Error,
            PotentialProblems::TooFewAuthorizedNames { .. } => Severity::Warn,

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
            PotentialProblems::IncompleteAddress { severity, .. } => *severity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct WithProblems(Vec<PotentialProblems>);

    impl Problematic for WithProblems {
        fn get_problems(&self) -> Vec<PotentialProblems> {
            self.0.clone()
        }
    }

    #[test]
    fn severity_order() {
        assert!(Severity::Info < Severity::Warn);
        assert!(Severity::Warn < Severity::Error);
    }

    #[test]
    fn highest_severity_none_when_no_problems() {
        let no_problems = WithProblems(vec![]);
        assert_eq!(no_problems.highest_severity(), None);
        assert_eq!(no_problems.highest_severity_class(), "ok");
        assert!(!no_problems.has_severity_or_higher(Severity::Info));
        assert!(!no_problems.has_severity_or_higher(Severity::Warn));
        assert!(!no_problems.has_severity_or_higher(Severity::Error));
        assert!(no_problems.is_all_good());
    }

    #[test]
    fn highest_severity_info_when_only_info() {
        let only_info = WithProblems(vec![PotentialProblems::NoLastName(Severity::Info)]);
        assert_eq!(only_info.highest_severity(), Some(Severity::Info));
        assert_eq!(only_info.highest_severity_class(), "info");
        assert!(only_info.has_severity_or_higher(Severity::Info));
        assert!(!only_info.has_severity_or_higher(Severity::Warn));
        assert!(!only_info.has_severity_or_higher(Severity::Error));
        assert!(!only_info.is_all_good());
    }

    #[test]
    fn highest_severity_warn_when_only_warnings() {
        let info_warn = WithProblems(vec![
            PotentialProblems::NoLastName(Severity::Info),
            PotentialProblems::NoLastName(Severity::Warn),
        ]);
        assert_eq!(info_warn.highest_severity(), Some(Severity::Warn));
        assert_eq!(info_warn.highest_severity_class(), "warning");
        assert!(info_warn.has_severity_or_higher(Severity::Info));
        assert!(info_warn.has_severity_or_higher(Severity::Warn));
        assert!(!info_warn.has_severity_or_higher(Severity::Error));
        assert!(!info_warn.is_all_good());
    }

    #[test]
    fn highest_severity_error_when_mix_of_severities() {
        let with_error = WithProblems(vec![
            PotentialProblems::NoLastName(Severity::Info),
            PotentialProblems::NoLastName(Severity::Warn),
            PotentialProblems::NoLastName(Severity::Error),
        ]);
        assert_eq!(with_error.highest_severity(), Some(Severity::Error));
        assert_eq!(with_error.highest_severity_class(), "error");
        assert!(with_error.has_severity_or_higher(Severity::Info));
        assert!(with_error.has_severity_or_higher(Severity::Warn));
        assert!(with_error.has_severity_or_higher(Severity::Error));
        assert!(!with_error.is_all_good());
    }

    #[test]
    fn no_problem_summary() {
        let no_problems = WithProblems(vec![]);
        assert_eq!(no_problems.problem_summary(&Locale::Nl), None);
    }

    #[test]
    fn single_problem_summary() {
        let problem = PotentialProblems::VeryOldDateOfBirth;
        let single_problems = WithProblems(vec![problem.clone()]);
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
        let multiple_problems = WithProblems(problems.to_vec());
        let summary = multiple_problems.problem_summary(&Locale::Nl).unwrap();
        for problem in problems {
            assert!(summary.contains(&problem.translate(&Locale::Nl)));
        }
    }

    #[test]
    fn deviation_shows_numbers() {
        let problems = vec![
            PotentialProblems::FewCandidatesWithFirstName {
                count: 2,
                total: 37,
            },
            PotentialProblems::FewCandidatesWithGender {
                count: 2,
                total: 37,
            },
            PotentialProblems::FewCandidatesWithoutFirstName {
                count: 2,
                total: 37,
            },
            PotentialProblems::FewCandidatesWithoutGender {
                count: 2,
                total: 37,
            },
            PotentialProblems::TooFewAuthorizedNames { actual: 2, min: 37 },
            PotentialProblems::TooManyAuthorizedNames { actual: 2, max: 37 },
        ];
        for problem in problems {
            let summary = WithProblems(vec![problem])
                .problem_summary(&Locale::Nl)
                .unwrap();
            assert!(summary.contains("2"));
            assert!(summary.contains("37"));
        }
    }
}
