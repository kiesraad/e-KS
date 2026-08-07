use crate::{
    AppError, CsbStore,
    structs::{candidate_lists::CandidateListId, persons::PersonId},
};

pub struct RestorationStatus {
    has_omissions: bool,
    has_corrections: bool,
}

impl RestorationStatus {
    pub fn for_political_group(store: &CsbStore) -> Self {
        RestorationStatus {
            has_omissions: !store.get_political_group_omissions().is_empty(),
            has_corrections: store.get_political_group_csb_corrections_count() > 0,
        }
    }

    pub fn for_declarations_of_support(store: &CsbStore) -> Self {
        RestorationStatus {
            has_omissions: !store.get_all_declarations_of_support_omissions().is_empty(),
            has_corrections: false,
        }
    }

    pub fn for_candidate_list(
        store: &CsbStore,
        list_id: CandidateListId,
    ) -> Result<Self, AppError> {
        Ok(RestorationStatus {
            has_omissions: store.has_candidate_list_omissions(list_id)?,
            has_corrections: store.has_candidate_list_csb_corrections(list_id)?,
        })
    }

    pub fn for_candidate(store: &CsbStore, person_id: PersonId, list_id: CandidateListId) -> Self {
        RestorationStatus {
            has_omissions: store.has_candidate_omissions(person_id, list_id),
            has_corrections: store.has_candidate_csb_corrections(person_id),
        }
    }

    pub fn has_omissions(&self) -> bool {
        self.has_omissions
    }

    pub fn has_corrections(&self) -> bool {
        self.has_corrections
    }
}
