use crate::id_newtype;

use chrono::{DateTime, Utc};

id_newtype!(pub struct PoliticalGroupId);

#[derive(Default, Debug, Clone)]
pub struct PoliticalGroup {
    pub id: PoliticalGroupId,

    pub long_list_allowed: Option<bool>,
    pub legal_name: Option<String>,
    pub display_name: Option<String>,

    #[allow(unused)]
    pub created_at: DateTime<Utc>,

    #[allow(unused)]
    pub updated_at: DateTime<Utc>,
}
