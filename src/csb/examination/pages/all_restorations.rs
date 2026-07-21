use askama::Template;
use axum::response::{IntoResponse, Response};
use axum_extra::routing::TypedPath;

use crate::{
    AppError, Context, CsbContext, CsbStore, ElectoralDistrict, HtmlTemplate, QueryParamState,
    candidate_lists::CandidateListId,
    csb::{
        Omission, OmissionCategory,
        examination::{extractors::CsbPoliticalGroup, pages::CsbAllRestorationsPath},
    },
    filters,
    persons::{Person, PersonId},
};

#[derive(Template)]
#[template(path = "csb/examination/pages/all_restorations.html")]
struct CsbAllRestorationsTemplate {
    political_group: CsbPoliticalGroup,
    restoration_count: usize,
    all_omissions: AllOmissions,
}

pub async fn all_restorations(
    _: CsbAllRestorationsPath,
    context: CsbContext,
    store: CsbStore,
) -> Result<Response, AppError> {
    let political_group = CsbPoliticalGroup::new_from_csb_store(&store);
    Ok(HtmlTemplate(
        CsbAllRestorationsTemplate {
            all_omissions: store.get_all_omissions(&political_group)?,
            political_group,
            restoration_count: store.get_omission_count(),
        },
        context,
    )
    .into_response())
}

struct AllOmissions {
    general: Vec<OmissionWithPath>,
    candidate_lists: Vec<OmissionWithPath>,
    candidates: Vec<CandidateOmissions>,
}

struct CandidateOmissions {
    omissions: Vec<OmissionWithPath>,
    person: Person,
}

struct OmissionWithPath {
    omission: Omission,
    path: String,
}

impl CsbStore {
    fn get_all_omissions(
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
        let mut candidate_lists = Vec::new();
        let mut candidates: Vec<CandidateOmissions> = Vec::new();

        for omission in omissions {
            match omission.category {
                OmissionCategory::PoliticalGroup => general.push(OmissionWithPath {
                    omission: omission.clone(),
                    path: general_path(political_group),
                }),
                OmissionCategory::CandidateList(ref districts) => {
                    candidate_lists.push(OmissionWithPath {
                        omission: omission.clone(),
                        path: list_path(political_group, districts, self)?,
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
                                .get_imported_or_corrected_person(person)
                                .ok_or(AppError::InternalServerError)?,
                        });
                    }
                }
            }
        }
        Ok(AllOmissions {
            general,
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

fn list_path(
    political_group: &CsbPoliticalGroup,
    districts: &[ElectoralDistrict],
    store: &CsbStore,
) -> Result<String, AppError> {
    let district = districts.first().ok_or(AppError::InternalServerError)?;
    // Search both projections: the omission may be recorded against a
    // district that only its imported or only its corrected list covers.
    let list = store
        .get_imported_candidate_lists()
        .into_iter()
        .chain(store.get_corrected_candidate_lists())
        .find(|l| l.electoral_districts.contains(district))
        .ok_or(AppError::InternalServerError)?
        .id;
    Ok(political_group
        .manage_candidate_list_omissions_path(&list)
        .with_query_params(QueryParamState::redirect_to(
            political_group.all_restorations_path().to_string(),
        ))
        .to_string())
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

    use reqwest::StatusCode;

    use crate::{
        StreamId,
        candidate_lists::CandidateList,
        common::UtcDateTime,
        csb::OmissionType,
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };

    use super::*;

    #[tokio::test]
    async fn all_restorations_shows_all_omissions() -> Result<(), AppError> {
        let store = CsbStore::new_for_test();
        let pg_title = "pg title".to_string();

        let candidate_title = "candidate title".to_string();
        let person_id = PersonId::new();
        store.add_person(sample_person(person_id));

        let list_title = "list title".to_string();
        let list_id = CandidateListId::new();
        store.add_candidate_list(CandidateList {
            id: list_id,
            electoral_districts: vec![ElectoralDistrict::UT, ElectoralDistrict::GR],
            candidates: vec![person_id],
            created_at: UtcDateTime::now(),
        });

        Omission::new(
            OmissionCategory::PoliticalGroup,
            pg_title.clone(),
            "description".to_string(),
            "help_text".to_string(),
        )
        .create(&store)
        .await?;

        Omission::new(
            OmissionCategory::CandidateList(vec![ElectoralDistrict::UT, ElectoralDistrict::GR]),
            list_title.clone(),
            "description".to_string(),
            "help_text".to_string(),
        )
        .create(&store)
        .await?;

        Omission::new(
            OmissionCategory::Candidate {
                person: person_id,
                lists: vec![list_id],
            },
            candidate_title.clone(),
            "description".to_string(),
            "help_text".to_string(),
        )
        .create(&store)
        .await?;

        let stream_id = store.stream_id;

        let response = all_restorations(
            CsbAllRestorationsPath { stream_id },
            CsbContext::new_test(),
            store,
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;

        // page contains titles
        assert!(body.contains(pg_title.as_str()));
        assert!(body.contains(list_title.as_str()));
        assert!(body.contains(candidate_title.as_str()));

        Ok(())
    }

    #[tokio::test]
    async fn all_restorations_shows_omissions_for_paper_added_candidates_and_lists()
    -> Result<(), AppError> {
        let store = CsbStore::new_for_test();

        // Both the candidate and the list only exist in the corrected
        // projection: they were added during paper corrections.
        let person = sample_person(PersonId::new());
        let person_id = person.id;
        store
            .data
            .write()
            .paper_corrected_data
            .persons
            .insert(person_id, person);
        let list_id = CandidateListId::new();
        store.set_paper_corrected_candidate_list(CandidateList {
            id: list_id,
            electoral_districts: vec![ElectoralDistrict::GR],
            candidates: vec![person_id],
            created_at: UtcDateTime::now(),
        });

        Omission::new(
            OmissionCategory::Candidate {
                person: person_id,
                lists: vec![list_id],
            },
            "candidate title".to_string(),
            "description".to_string(),
            "help_text".to_string(),
        )
        .create(&store)
        .await?;

        Omission::new(
            OmissionCategory::CandidateList(vec![ElectoralDistrict::GR]),
            "list title".to_string(),
            "description".to_string(),
            "help_text".to_string(),
        )
        .create(&store)
        .await?;

        let response = all_restorations(
            CsbAllRestorationsPath {
                stream_id: store.stream_id,
            },
            CsbContext::new_test(),
            store,
        )
        .await?;

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("candidate title"));
        assert!(body.contains("list title"));

        Ok(())
    }

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
    fn list_path_test() {
        let store = CsbStore::new_for_test();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        store.add_candidate_list(list.clone());

        let path = list_path(
            &CsbPoliticalGroup::new_from_csb_store(&store),
            &list.electoral_districts,
            &store,
        )
        .expect("list path can't be created");

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
