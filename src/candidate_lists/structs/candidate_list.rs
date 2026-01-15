use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;
use uuid::Uuid;

use crate::{AppError, ElectionConfig, ElectoralDistrict, Locale, candidate_lists::Candidate, t};

/// Maximum number of persons allowed on a candidate list.
pub const MAX_CANDIDATES: usize = 50;

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type, PartialEq, Eq)]
pub struct CandidateList {
    pub id: Uuid,
    pub electoral_districts: Vec<ElectoralDistrict>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CandidateListSummary {
    pub list: CandidateList,
    pub person_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FullCandidateList {
    pub list: CandidateList,
    pub candidates: Vec<Candidate>,
}

impl CandidateList {
    pub fn display_districts(&self, election: &ElectionConfig, locale: &Locale) -> String {
        if !self.electoral_districts.is_empty()
            && self.electoral_districts.len() == election.electoral_districts().len()
        {
            t!("candidate_list.districts.all", locale).to_string()
        } else {
            self.electoral_districts
                .iter()
                .map(|d| d.title())
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

impl FullCandidateList {
    pub fn get_index(&self, person_id: &Uuid) -> Option<usize> {
        self.candidates
            .iter()
            .position(|c| &c.person.id == person_id)
    }

    pub fn get_candidate(&self, person_id: &Uuid, locale: Locale) -> Result<Candidate, AppError> {
        self.candidates
            .iter()
            .find(|c| &c.person.id == person_id)
            .cloned()
            .ok_or_else(|| {
                AppError::NotFound(t!("person.not_found_in_candidate_list", &locale).to_string())
            })
    }

    pub fn get_ids(&self) -> Vec<Uuid> {
        self.candidates.iter().map(|c| c.person.id).collect()
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn display_districts_returns_all_for_full_set() {
        let list = CandidateList {
            id: Uuid::new_v4(),
            electoral_districts: ElectionConfig::EK2027.electoral_districts().to_vec(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(
            list.display_districts(&ElectionConfig::EK2027, &Locale::Nl),
            "Alle"
        );
    }

    #[test]
    fn display_districts_returns_titles_for_subset() {
        let list = CandidateList {
            id: Uuid::new_v4(),
            electoral_districts: vec![ElectoralDistrict::UT, ElectoralDistrict::DR],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(
            list.display_districts(&ElectionConfig::EK2027, &Locale::Nl),
            "Utrecht, Drenthe"
        );
    }
}
