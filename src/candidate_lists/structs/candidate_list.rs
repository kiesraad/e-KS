use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;

use crate::{ElectoralDistrict, id_newtype};

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
