use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;

use crate::{ElectionConfig, ElectoralDistrict, Locale, id_newtype, t};

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
    use super::*;

    #[test]
    fn display_districts_returns_all_for_full_set() {
        let list = CandidateList {
            id: CandidateListId::new(),
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
            id: CandidateListId::new(),
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
