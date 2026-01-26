use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;

use crate::{ElectionConfig, ElectoralDistrict, id_newtype};

id_newtype!(pub struct CandidateListId);

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type, PartialEq, Eq)]
pub struct CandidateList {
    pub id: CandidateListId,
    pub electoral_districts: Vec<ElectoralDistrict>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CandidateListSummary {
    pub list: CandidateList,
    pub person_count: i64,
}

impl CandidateList {
    pub fn districts(&self) -> String {
        self.electoral_districts
            .iter()
            .map(|d| d.title())
            .collect::<Vec<&str>>()
            .join(", ")
    }

    pub fn contains_all_districts(&self, election: &ElectionConfig) -> bool {
        self.electoral_districts.len() == election.electoral_districts().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::types::chrono::Utc;

    fn base_candidate_list(electoral_districts: Vec<ElectoralDistrict>) -> CandidateList {
        CandidateList {
            id: CandidateListId::new(),
            electoral_districts,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn districts_formats_titles_in_order() {
        let list = base_candidate_list(vec![
            ElectoralDistrict::UT,
            ElectoralDistrict::NH,
            ElectoralDistrict::DR,
        ]);

        assert_eq!(list.districts(), "Utrecht, Noord-Holland, Drenthe");
    }

    #[test]
    fn contains_all_districts_compares_to_election_config_length() {
        let election = ElectionConfig::EK2027;
        let list = base_candidate_list(election.electoral_districts().to_vec());
        assert!(list.contains_all_districts(&election));

        let list = base_candidate_list(vec![ElectoralDistrict::UT, ElectoralDistrict::NH]);
        assert!(!list.contains_all_districts(&election));
    }
}
