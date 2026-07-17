use askama::Template;
use axum::response::{IntoResponse, Response};
use axum_extra::routing::TypedPath;

use crate::{
    AppError, Context, CsbContext, CsbStore, ElectoralDistrict, HtmlTemplate, QueryParamState,
    candidate_lists::CandidateListId,
    csb::{
        Omission,
        OmissionCategory::{Candidate, CandidateList, PoliticalGroup},
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
                PoliticalGroup => general.push(OmissionWithPath {
                    omission: omission.clone(),
                    path: general_path(political_group),
                }),
                CandidateList(ref districts) => candidate_lists.push(OmissionWithPath {
                    omission: omission.clone(),
                    path: list_path(political_group, districts, self)?,
                }),
                Candidate { person, ref lists } => {
                    if let Some(candidate) = candidates.iter_mut().find(|c| c.person.id == person) {
                        candidate.omissions.push(OmissionWithPath {
                            path: candidate_path(political_group, &person, &lists[0]),
                            omission: omission.clone(),
                        })
                    } else {
                        candidates.push(CandidateOmissions {
                            omissions: vec![OmissionWithPath {
                                path: candidate_path(political_group, &person, &lists[0]),
                                omission,
                            }],
                            person: self
                                .get_person(person)
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
    let list = store
        .get_candidate_lists()
        .iter()
        .find(|l| l.electoral_districts.contains(&districts[0]))
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
    // TODO
}
