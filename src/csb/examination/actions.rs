//! Store-backed operations for omissions.

use crate::{
    AnyLocale, AppError, CsbEvent, CsbStore, ElectionConfig, ElectoralDistrict,
    structs::csb::{Omission, OmissionCategory},
};

const ALL_DISTRICTS: &str = "alle kieskringen";

impl OmissionCategory {
    /// Returns the electoral district string for use in model I 4.
    pub fn electoral_district(
        &self,
        store: &CsbStore,
        election: &ElectionConfig,
    ) -> Result<String, AppError> {
        match self {
            OmissionCategory::PoliticalGroup => Ok(ALL_DISTRICTS.to_string()),
            OmissionCategory::CandidateList(lists) | OmissionCategory::Candidate { lists, .. } => {
                let mut districts: Vec<ElectoralDistrict> = Vec::new();
                for id in lists {
                    let list = store
                        .get_candidate_list(*id, crate::projection::WithCorrections::All)
                        .ok_or(AppError::GenericNotFound)?;
                    for d in list.electoral_districts {
                        if !districts.contains(&d) {
                            districts.push(d);
                        }
                    }
                }
                Ok(format_districts(&districts, election))
            }
            OmissionCategory::DeclarationsOfSupport(districts) => {
                Ok(format_districts(districts, election))
            }
        }
    }
}

fn format_districts(districts: &[ElectoralDistrict], election: &ElectionConfig) -> String {
    let all_districts = election.electoral_districts();
    if districts.is_empty() || all_districts.iter().all(|d| districts.contains(d)) {
        ALL_DISTRICTS.to_string()
    } else {
        let mut sorted = districts.to_vec();
        sorted.sort_by_key(|d| d.region_number());
        let parts: Vec<String> = sorted
            .iter()
            .map(|d| format!("{} ({})", d.region_number(), d.title(AnyLocale::Nl)))
            .collect();
        format!("kieskring {}", parts.join(", "))
    }
}

impl Omission {
    pub async fn create(&self, store: &CsbStore) -> Result<(), AppError> {
        store.update(CsbEvent::CreateOmission(self.clone())).await
    }

    pub async fn update(&self, store: &CsbStore) -> Result<(), AppError> {
        store.update(CsbEvent::UpdateOmission(self.clone())).await
    }

    pub async fn delete(&self, store: &CsbStore) -> Result<(), AppError> {
        store
            .update(CsbEvent::DeleteOmission {
                omission_id: self.id,
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ElectionConfig, ElectoralDistrict,
        structs::candidate_lists::{CandidateList, CandidateListId},
    };

    const EK: ElectionConfig = ElectionConfig::EK27;

    fn store_with_list(districts: Vec<ElectoralDistrict>) -> (CsbStore, CandidateListId) {
        let store = CsbStore::new_for_test();
        let list = CandidateList {
            electoral_districts: districts,
            ..Default::default()
        };
        let id = list.id;
        store.add_candidate_list(list);
        (store, id)
    }

    #[test]
    fn political_group_maps_to_all_districts() {
        let store = CsbStore::new_for_test();
        assert_eq!(
            OmissionCategory::PoliticalGroup
                .electoral_district(&store, &EK)
                .unwrap(),
            "alle kieskringen"
        );
    }

    #[test]
    fn candidate_with_all_districts_maps_to_all() {
        let (store, id) = store_with_list(EK.electoral_districts().to_vec());
        let category = OmissionCategory::Candidate {
            person: crate::structs::persons::PersonId::new(),
            lists: vec![id],
        };
        assert_eq!(
            category.electoral_district(&store, &EK).unwrap(),
            "alle kieskringen"
        );
    }

    #[test]
    fn dos_all_districts_maps_to_all() {
        let store = CsbStore::new_for_test();
        assert_eq!(
            OmissionCategory::DeclarationsOfSupport(EK.electoral_districts().to_vec())
                .electoral_district(&store, &EK)
                .unwrap(),
            "alle kieskringen"
        );
    }

    #[test]
    fn dos_one_district() {
        let store = CsbStore::new_for_test();
        assert_eq!(
            OmissionCategory::DeclarationsOfSupport(vec![ElectoralDistrict::BO])
                .electoral_district(&store, &EK)
                .unwrap(),
            "kieskring 13 (Bonaire)"
        );
    }

    #[test]
    fn dos_multiple_districts() {
        let store = CsbStore::new_for_test();
        // The districts should be sorted by region number.
        assert_eq!(
            OmissionCategory::DeclarationsOfSupport(vec![
                ElectoralDistrict::DR,
                ElectoralDistrict::GR
            ])
            .electoral_district(&store, &EK)
            .unwrap(),
            "kieskring 1 (Groningen), 3 (Drenthe)"
        );
    }

    #[test]
    fn dos_no_districts_maps_to_all() {
        let store = CsbStore::new_for_test();
        // An empty district list is treated as "all districts" in format_districts.
        assert_eq!(
            OmissionCategory::DeclarationsOfSupport(vec![])
                .electoral_district(&store, &EK)
                .unwrap(),
            "alle kieskringen"
        );
    }

    #[test]
    fn candidate_with_list_specific_district() {
        let (store, id) = store_with_list(vec![ElectoralDistrict::GR]);
        let category = OmissionCategory::Candidate {
            person: crate::structs::persons::PersonId::new(),
            lists: vec![id],
        };
        assert_eq!(
            category.electoral_district(&store, &EK).unwrap(),
            "kieskring 1 (Groningen)"
        );
    }

    #[test]
    fn candidate_with_paper_added_list_uses_the_corrected_projection() {
        let store = CsbStore::new_for_test();
        let list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::GR],
            ..Default::default()
        };
        let id = list.id;
        store.set_paper_corrected_candidate_list(list);
        let category = OmissionCategory::Candidate {
            person: crate::structs::persons::PersonId::new(),
            lists: vec![id],
        };

        assert_eq!(
            category.electoral_district(&store, &EK).unwrap(),
            "kieskring 1 (Groningen)"
        );
    }

    #[test]
    fn candidate_with_corrected_list_uses_the_corrected_districts() {
        let (store, id) = store_with_list(vec![ElectoralDistrict::UT]);
        store.set_paper_corrected_candidate_list(CandidateList {
            id,
            electoral_districts: vec![ElectoralDistrict::GR],
            ..Default::default()
        });
        let category = OmissionCategory::Candidate {
            person: crate::structs::persons::PersonId::new(),
            lists: vec![id],
        };

        assert_eq!(
            category.electoral_district(&store, &EK).unwrap(),
            "kieskring 1 (Groningen)"
        );
    }

    #[test]
    fn candidate_with_missing_list_returns_error() {
        let store = CsbStore::new_for_test();
        let category = OmissionCategory::Candidate {
            person: crate::structs::persons::PersonId::new(),
            lists: vec![CandidateListId::new()],
        };
        assert!(category.electoral_district(&store, &EK).is_err());
    }
}
