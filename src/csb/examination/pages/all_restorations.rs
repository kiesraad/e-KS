use askama::Template;
use axum::response::{IntoResponse, Response};
use axum_extra::routing::TypedPath;

use crate::{
    AppError, Context, CsbContext, CsbStore, HtmlTemplate, QueryParamState,
    csb::{
        Omission,
        OmissionCategory::{
            Candidate, CandidateList, DeclarationOfSupport, General, NameAuthorisation,
        },
        examination::{
            extractors::CsbPoliticalGroup,
            pages::CsbAllRestorationsPath,
        },
    },
    filters,
    persons::Person,
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
            political_group,
            restoration_count: store.get_omission_count(),
            all_omissions: store.get_all_omissions()?,
        },
        context,
    )
    .into_response())
}

struct AllOmissions {
    general: Vec<Omission>,
    candidate_lists: Vec<Omission>,
    candidates: Vec<CandidateOmissions>,
}

struct CandidateOmissions {
    omissions: Vec<Omission>,
    person: Person,
}

impl CsbStore {
    fn get_all_omissions(&self) -> Result<AllOmissions, AppError> {
        let omissions = self
            .data
            .read()
            .omissions
            .values()
            .cloned()
            .collect::<Vec<_>>();

        let mut general = Vec::new();
        let mut candidate_lists = Vec::new();

        for o in omissions {
            match &o.category {
                General => general.push(o),
                CandidateList(_) => candidate_lists.push(o),
                Candidate { .. } => {} // candidate omissions collected separately
                NameAuthorisation(_) | DeclarationOfSupport(_) => {
                    todo!("remove after merge of #965")
                }
            }
        }

        let mut candidates = Vec::new();

        for person_id in self.get_candidates_with_omissions() {
            candidates.push(CandidateOmissions {
                omissions: self.get_candidate_omissions(person_id),
                person: self
                    .get_person(person_id)
                    .ok_or(AppError::InternalServerError)?,
            });
        }
        Ok(AllOmissions {
            general,
            candidate_lists,
            candidates,
        })
    }
}

impl Omission {
    fn path(
        &self,
        political_group: &CsbPoliticalGroup
    ) -> String {
        match self.category {
            General => political_group
                .manage_political_group_omissions_path()
                .with_query_params(QueryParamState::redirect_to(
                    political_group.all_restorations_path().to_string(),
                ))
                .to_string(),
            CandidateList(list_id) => political_group
                .manage_candidate_list_omissions_path(&list_id)
                .with_query_params(QueryParamState::redirect_to(
                    political_group.all_restorations_path().to_string(),
                ))
                .to_string(),
            Candidate {
                person,
                list: Some(list_id),
            } => political_group
                .manage_candidate_omissions_path(&person, &list_id)
                .with_query_params(QueryParamState::redirect_to(
                    political_group.all_restorations_path().to_string(),
                ))
                .to_string(),
            Candidate { person: _, list: None } =>
            // TODO: solve this after omission rework
            "".to_string(),
            NameAuthorisation(_) | DeclarationOfSupport(_) => todo!("remove after merge of #965"),
        }
    }
}
