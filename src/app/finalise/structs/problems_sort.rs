use std::cmp;

use crate::{
    common::Severity,
    finalise::{AllProblems, EntityProblems},
};

impl AllProblems {
    pub fn sort_problems_by_severity(&mut self) {
        // general sorting
        self.general
            .general
            .sort_by_key(|p| cmp::Reverse(p.severity()));
        self.general
            .name_authorisations
            .iter_mut()
            .for_each(|ep| ep.problems.sort_by_key(|p| cmp::Reverse(p.severity())));
        if let Some(ls) = self.general.list_submitter.as_mut() {
            ls.problems.sort_by_key(|p| cmp::Reverse(p.severity()))
        }
        self.general
            .substitute_submitters
            .iter_mut()
            .for_each(|ss| ss.problems.sort_by_key(|p| cmp::Reverse(p.severity())));

        // candidate sorting
        self.candidates
            .iter_mut()
            .for_each(|c| c.problems.sort_by_key(|p| cmp::Reverse(p.severity())));

        // list sorting
        self.lists
            .general
            .sort_by_key(|p| cmp::Reverse(p.severity()));

        let mut list_error_problems = Vec::new();
        let mut list_other_problems = Vec::new();
        self.lists.per_list.iter().for_each(|l| {
            list_error_problems.push(EntityProblems {
                entity: l.entity.clone(),
                problems: l
                    .problems
                    .iter()
                    .filter(|p| p.severity() == Severity::Error)
                    .cloned()
                    .collect(),
            });
            list_other_problems.push(EntityProblems {
                entity: l.entity.clone(),
                problems: l
                    .problems
                    .iter()
                    .filter(|p| p.severity() != Severity::Error)
                    .cloned()
                    .collect(),
            });
        });
        list_error_problems.extend(list_other_problems);
        list_error_problems.retain(|p| !p.problems.is_empty());
        self.lists.per_list = list_error_problems;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        candidate_lists::CandidateListId,
        common::{
            InfoProblems::{self},
            PotentialProblems,
        },
        finalise::{
            EntityProblems, GeneralProblems, ListProblems, PersonProblems,
            structs::problems::EntityInfoProblems,
        },
        list_submitters::ListSubmitterId,
        name_authorisations::NameAuthorisationId,
        persons::PersonId,
        test_utils::{
            sample_candidate_list, sample_list_submitter, sample_name_authorisation, sample_person,
        },
    };

    use super::*;

    fn problem_vec_unordered() -> Vec<PotentialProblems> {
        vec![
            PotentialProblems::NoInitials(Severity::Warn),
            PotentialProblems::NoInitials(Severity::Error),
        ]
    }

    fn problem_vec_ordered() -> Vec<PotentialProblems> {
        vec![
            PotentialProblems::NoInitials(Severity::Error),
            PotentialProblems::NoInitials(Severity::Warn),
        ]
    }

    #[test]
    fn sort_by_severity() {
        let nn1 = sample_name_authorisation(NameAuthorisationId::new());
        let nn2 = sample_name_authorisation(NameAuthorisationId::new());

        let ls = sample_list_submitter(ListSubmitterId::new());

        let ss1 = sample_list_submitter(ListSubmitterId::new());
        let ss2 = sample_list_submitter(ListSubmitterId::new());

        let c1 = sample_person(PersonId::new());
        let c2 = sample_person(PersonId::new());

        let cl1 = sample_candidate_list(CandidateListId::new());
        let cl2 = sample_candidate_list(CandidateListId::new());

        let info_problems = vec![
            EntityInfoProblems::Person {
                person: Box::new(sample_person(PersonId::new())),
                problem: InfoProblems::NoLastName,
            },
            EntityInfoProblems::Person {
                person: Box::new(sample_person(PersonId::new())),
                problem: InfoProblems::NoInitials,
            },
        ];

        let mut all_problems = AllProblems {
            general: GeneralProblems {
                general: problem_vec_unordered(),
                name_authorisations: vec![
                    EntityProblems {
                        entity: nn1.clone(),
                        problems: problem_vec_unordered(),
                    },
                    EntityProblems {
                        entity: nn2.clone(),
                        problems: problem_vec_unordered(),
                    },
                ],
                list_submitter: Some(EntityProblems {
                    entity: ls.clone(),
                    problems: problem_vec_unordered(),
                }),
                substitute_submitters: vec![
                    EntityProblems {
                        entity: ss1.clone(),
                        problems: problem_vec_unordered(),
                    },
                    EntityProblems {
                        entity: ss2.clone(),
                        problems: problem_vec_unordered(),
                    },
                ],
            },
            candidates: vec![
                PersonProblems {
                    entity: c1.clone(),
                    problems: problem_vec_unordered(),
                },
                PersonProblems {
                    entity: c2.clone(),
                    problems: problem_vec_unordered(),
                },
            ],
            lists: ListProblems {
                general: problem_vec_unordered(),
                per_list: vec![
                    EntityProblems {
                        entity: cl1.clone(),
                        problems: problem_vec_unordered(),
                    },
                    EntityProblems {
                        entity: cl2.clone(),
                        problems: problem_vec_unordered(),
                    },
                ],
            },
            info_problems: info_problems.clone(),
        };

        all_problems.sort_problems_by_severity();

        assert_eq!(
            all_problems,
            AllProblems {
                general: GeneralProblems {
                    general: problem_vec_ordered(),
                    name_authorisations: vec![
                        EntityProblems {
                            entity: nn1,
                            problems: problem_vec_ordered(),
                        },
                        EntityProblems {
                            entity: nn2,
                            problems: problem_vec_ordered(),
                        },
                    ],
                    list_submitter: Some(EntityProblems {
                        entity: ls,
                        problems: problem_vec_ordered(),
                    }),
                    substitute_submitters: vec![
                        EntityProblems {
                            entity: ss1,
                            problems: problem_vec_ordered(),
                        },
                        EntityProblems {
                            entity: ss2,
                            problems: problem_vec_ordered(),
                        },
                    ],
                },
                candidates: vec![
                    PersonProblems {
                        entity: c1,
                        problems: problem_vec_ordered(),
                    },
                    PersonProblems {
                        entity: c2,
                        problems: problem_vec_ordered(),
                    },
                ],
                lists: ListProblems {
                    general: problem_vec_ordered(),
                    per_list: vec![
                        EntityProblems {
                            entity: cl1.clone(),
                            problems: vec![PotentialProblems::NoInitials(Severity::Error)],
                        },
                        EntityProblems {
                            entity: cl2.clone(),
                            problems: vec![PotentialProblems::NoInitials(Severity::Error)],
                        },
                        EntityProblems {
                            entity: cl1.clone(),
                            problems: vec![PotentialProblems::NoInitials(Severity::Warn)],
                        },
                        EntityProblems {
                            entity: cl2.clone(),
                            problems: vec![PotentialProblems::NoInitials(Severity::Warn)],
                        },
                    ],
                },
                info_problems,
            }
        )
    }

    #[test]
    fn sort_by_severity_is_idempotent() {
        let nn1 = sample_name_authorisation(NameAuthorisationId::new());
        let nn2 = sample_name_authorisation(NameAuthorisationId::new());

        let ls = sample_list_submitter(ListSubmitterId::new());

        let ss1 = sample_list_submitter(ListSubmitterId::new());
        let ss2 = sample_list_submitter(ListSubmitterId::new());

        let c1 = sample_person(PersonId::new());
        let c2 = sample_person(PersonId::new());

        let cl1 = sample_candidate_list(CandidateListId::new());
        let cl2 = sample_candidate_list(CandidateListId::new());

        let info_problems = vec![
            EntityInfoProblems::Person {
                person: Box::new(sample_person(PersonId::new())),
                problem: InfoProblems::NoLastName,
            },
            EntityInfoProblems::Person {
                person: Box::new(sample_person(PersonId::new())),
                problem: InfoProblems::NoInitials,
            },
        ];

        let mut all_problems = AllProblems {
            general: GeneralProblems {
                general: problem_vec_unordered(),
                name_authorisations: vec![
                    EntityProblems {
                        entity: nn1.clone(),
                        problems: problem_vec_unordered(),
                    },
                    EntityProblems {
                        entity: nn2.clone(),
                        problems: problem_vec_unordered(),
                    },
                ],
                list_submitter: Some(EntityProblems {
                    entity: ls.clone(),
                    problems: problem_vec_unordered(),
                }),
                substitute_submitters: vec![
                    EntityProblems {
                        entity: ss1.clone(),
                        problems: problem_vec_unordered(),
                    },
                    EntityProblems {
                        entity: ss2.clone(),
                        problems: problem_vec_unordered(),
                    },
                ],
            },
            candidates: vec![
                PersonProblems {
                    entity: c1.clone(),
                    problems: problem_vec_unordered(),
                },
                PersonProblems {
                    entity: c2.clone(),
                    problems: problem_vec_unordered(),
                },
            ],
            lists: ListProblems {
                general: problem_vec_unordered(),
                per_list: vec![
                    EntityProblems {
                        entity: cl1.clone(),
                        problems: problem_vec_unordered(),
                    },
                    EntityProblems {
                        entity: cl2.clone(),
                        problems: problem_vec_unordered(),
                    },
                ],
            },
            info_problems: info_problems.clone(),
        };

        let original = all_problems.clone();

        all_problems.sort_problems_by_severity();

        let after_sort1 = all_problems.clone();

        all_problems.sort_problems_by_severity();

        let after_sort2 = all_problems;

        assert_ne!(original, after_sort1);
        assert_eq!(after_sort1, after_sort2);
    }
}
