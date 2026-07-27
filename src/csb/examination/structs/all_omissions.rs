use axum_extra::routing::TypedPath;

use crate::{
    AppError, CsbStore, ElectoralDistrict, QueryParamState,
    candidate_lists::CandidateListId,
    csb::examination::extractors::CsbPoliticalGroup,
    persons::{Person, PersonId},
    structs::csb::{Omission, OmissionCategory},
};

pub struct AllOmissions {
    pub general: Vec<OmissionWithPath>,
    pub declarations_of_support: Vec<OmissionWithPath>,
    pub candidate_lists: Vec<OmissionWithPath>,
    pub candidates: Vec<CandidateOmissions>,
}

pub struct CandidateOmissions {
    pub omissions: Vec<OmissionWithPath>,
    pub person: Person,
}

pub struct OmissionWithPath {
    pub omission: Omission,
    pub path: String,
}

impl CsbStore {
    pub fn get_all_omissions(
        &self,
        political_group: &CsbPoliticalGroup,
    ) -> Result<AllOmissions, AppError> {
        let omissions = self
            .data
            .read()
            .omissions
            .values()
            .cloned()
            .collect::<Vec<_>>();

        let mut general = Vec::new();
        let mut declarations_of_support = Vec::new();
        let mut candidate_lists = Vec::new();
        let mut candidates: Vec<CandidateOmissions> = Vec::new();

        for omission in omissions {
            match omission.category {
                OmissionCategory::PoliticalGroup => general.push(OmissionWithPath {
                    omission: omission.clone(),
                    path: general_path(political_group),
                }),
                OmissionCategory::CandidateList(ref lists) => {
                    let list_id = lists.first().ok_or(AppError::InternalServerError)?;
                    candidate_lists.push(OmissionWithPath {
                        omission: omission.clone(),
                        path: political_group
                            .manage_candidate_list_omissions_path(list_id)
                            .with_query_params(QueryParamState::redirect_to(
                                political_group.all_restorations_path().to_string(),
                            ))
                            .to_string(),
                    })
                }
                OmissionCategory::DeclarationsOfSupport(_) => {
                    declarations_of_support.push(OmissionWithPath {
                        omission: omission.clone(),
                        path: political_group
                            .manage_declarations_of_support_omissions_path()
                            .with_query_params(QueryParamState::redirect_to(
                                political_group.all_restorations_path().to_string(),
                            ))
                            .to_string(),
                    })
                }
                OmissionCategory::Candidate { person, ref lists } => {
                    let list = lists.first().ok_or(AppError::InternalServerError)?;
                    if let Some(candidate) = candidates.iter_mut().find(|c| c.person.id == person) {
                        candidate.omissions.push(OmissionWithPath {
                            path: candidate_path(political_group, &person, list),
                            omission: omission.clone(),
                        })
                    } else {
                        candidates.push(CandidateOmissions {
                            omissions: vec![OmissionWithPath {
                                path: candidate_path(political_group, &person, list),
                                omission,
                            }],
                            person: self
                                .get_person(person, crate::csb::WithCorrections::All)
                                .ok_or(AppError::InternalServerError)?,
                        });
                    }
                }
            }
        }
        Ok(AllOmissions {
            general,
            declarations_of_support,
            candidate_lists,
            candidates,
        })
    }
}

fn general_path(political_group: &CsbPoliticalGroup) -> String {
    political_group
        .manage_political_group_omissions_path()
        .with_query_params(QueryParamState::redirect_to(
            political_group.all_restorations_path().to_string(),
        ))
        .to_string()
}

fn candidate_path(
    political_group: &CsbPoliticalGroup,
    person: &PersonId,
    list: &CandidateListId,
) -> String {
    political_group
        .manage_candidate_omissions_path(person, list)
        .with_query_params(QueryParamState::redirect_to(
            political_group.all_restorations_path().to_string(),
        ))
        .to_string()
}

#[cfg(test)]
mod tests {
    use crate::{
        StreamId,
        structs::csb::OmissionType,
        test_utils::{sample_candidate_list, sample_person},
    };

    use super::*;

    fn redirect_param(stream_id: StreamId) -> String {
        format!("&redirect_to=%2Fcsb%2Fexamination%2F{stream_id}%2Fomissions")
    }

    #[test]
    fn general_path_test() {
        let store = CsbStore::new_for_test();

        let path = general_path(&CsbPoliticalGroup::new_from_csb_store(&store));

        let pg_type = OmissionType::PoliticalGroup.to_string();
        let stream_id = store.stream_id;
        let redirect_param = redirect_param(stream_id);
        assert!(
            path.contains(
                format!("/csb/examination/{stream_id}/omission/{pg_type}/{stream_id}/overview?")
                    .as_str()
            )
        );
        assert!(path.contains(redirect_param.as_str()));
    }

    #[test]
    fn candidate_list_omission_path_links_to_the_referenced_list() {
        let store = CsbStore::new_for_test();
        let list_id = CandidateListId::new();
        store.add_candidate_list(sample_candidate_list(list_id));

        let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
        let path = political_group
            .manage_candidate_list_omissions_path(&list_id)
            .with_query_params(QueryParamState::redirect_to(
                political_group.all_restorations_path().to_string(),
            ))
            .to_string();

        let list_type = OmissionType::CandidateList.to_string();
        let stream_id = store.stream_id;
        let redirect_param = redirect_param(stream_id);
        assert!(
            path.contains(
                format!("/csb/examination/{stream_id}/omission/{list_type}/{list_id}/overview?")
                    .as_str()
            )
        );
        assert!(path.contains(redirect_param.as_str()));
    }

    #[test]
    fn candidate_path_test() {
        let store = CsbStore::new_for_test();

        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        store.add_candidate_list(list);

        let person_id = PersonId::new();
        store.add_person(sample_person(person_id));

        let path = candidate_path(
            &CsbPoliticalGroup::new_from_csb_store(&store),
            &person_id,
            &list_id,
        );

        let candidate_type = OmissionType::Candidate.to_string();
        let stream_id = store.stream_id;
        let redirect_param = redirect_param(stream_id);
        let list_param = format!("&list={list_id}");
        assert!(
            path.contains(
                format!(
                    "/csb/examination/{stream_id}/omission/{candidate_type}/{person_id}/overview?"
                )
                .as_str()
            )
        );
        assert!(path.contains(list_param.as_str()));
        assert!(path.contains(redirect_param.as_str()));
    }

    #[test]
    fn person_without_omissions() {
        let store = CsbStore::new_for_test();

        store.add_person(sample_person(PersonId::new()));

        let all_omissions = store
            .get_all_omissions(&CsbPoliticalGroup::new_from_csb_store(&store))
            .expect("Couldn't retrieve all omissions");

        assert!(all_omissions.candidates.is_empty());
    }

    #[tokio::test]
    async fn person_with_omission() {
        let store = CsbStore::new_for_test();

        let person_id = PersonId::new();
        store.add_person(sample_person(person_id));

        let list_id = CandidateListId::new();
        store.add_candidate_list(sample_candidate_list(list_id));

        Omission::new(
            OmissionCategory::Candidate {
                person: person_id,
                lists: vec![list_id],
            },
            "title".to_string(),
            "description".to_string(),
            "help_text".to_string(),
        )
        .create(&store)
        .await
        .expect("Couldn't create omission");

        let all_omissions = store
            .get_all_omissions(&CsbPoliticalGroup::new_from_csb_store(&store))
            .expect("Couldn't retrieve all omissions");

        assert_eq!(all_omissions.candidates.len(), 1);
        assert_eq!(all_omissions.candidates[0].omissions.len(), 1)
    }

    #[tokio::test]
    async fn person_with_multiple_omissions() {
        let omission_count = 10;
        let store = CsbStore::new_for_test();

        let person_id = PersonId::new();
        store.add_person(sample_person(person_id));

        let list_id = CandidateListId::new();
        store.add_candidate_list(sample_candidate_list(list_id));
        for _ in 0..omission_count {
            Omission::new(
                OmissionCategory::Candidate {
                    person: person_id,
                    lists: vec![list_id],
                },
                "title".to_string(),
                "description".to_string(),
                "help_text".to_string(),
            )
            .create(&store)
            .await
            .expect("Couldn't create omission");
        }

        let all_omissions = store
            .get_all_omissions(&CsbPoliticalGroup::new_from_csb_store(&store))
            .expect("Couldn't retrieve all omissions");

        // creates one candidate with 10 omissions
        assert_eq!(all_omissions.candidates.len(), 1);
        assert_eq!(all_omissions.candidates[0].omissions.len(), omission_count)
    }
}
