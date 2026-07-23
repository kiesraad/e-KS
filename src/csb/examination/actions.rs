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
            OmissionCategory::CandidateList(districts) => Ok(format_districts(districts, election)),
            OmissionCategory::Candidate { lists, .. } => {
                let mut districts: Vec<ElectoralDistrict> = Vec::new();
                for id in lists {
                    let list = store
                        .get_candidate_list(*id)
                        .ok_or(AppError::GenericNotFound)?;
                    for d in list.electoral_districts {
                        if !districts.contains(&d) {
                            districts.push(d);
                        }
                    }
                }
                Ok(format_districts(&districts, election))
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
