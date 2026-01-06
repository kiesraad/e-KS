use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;
use uuid::Uuid;

use crate::{ElectionConfig, ElectoralDistrict, Locale, persons::structs::Person, t};

/// Maximum number of persons allowed on a candidate list.
pub const MAX_CANDIDATES: usize = 80;

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type)]
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
pub struct CandidateListEntry {
    pub position: i32,
    pub person: Person,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateListDetail {
    pub list: CandidateList,
    pub candidates: Vec<CandidateListEntry>,
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
