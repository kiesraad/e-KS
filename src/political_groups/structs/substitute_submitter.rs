use crate::id_newtype;
use chrono::{DateTime, Utc};

id_newtype!(pub struct SubstituteSubmitterId);

#[derive(Default, Debug, Clone)]
pub struct SubstituteSubmitter {
    pub id: SubstituteSubmitterId,

    pub last_name: String,
    pub last_name_prefix: Option<String>,
    pub initials: String,

    pub locality: Option<String>,
    pub postal_code: Option<String>,
    pub house_number: Option<String>,
    pub house_number_addition: Option<String>,
    pub street_name: Option<String>,

    #[allow(unused)]
    pub created_at: DateTime<Utc>,
    #[allow(unused)]
    pub updated_at: DateTime<Utc>,
}

impl SubstituteSubmitter {
    pub fn last_name_with_prefix(&self) -> String {
        if let Some(prefix) = &self.last_name_prefix {
            format!("{} {}", prefix, self.last_name)
        } else {
            self.last_name.clone()
        }
    }

    pub fn is_address_complete(&self) -> bool {
        self.street_name.is_some()
            && self.house_number.is_some()
            && self.postal_code.is_some()
            && self.locality.is_some()
    }
}
