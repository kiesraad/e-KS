use crate::id_newtype;
use chrono::{DateTime, Utc};

id_newtype!(pub struct AuthorisedAgentId);

#[derive(Debug, Clone)]
pub struct AuthorisedAgent {
    pub id: AuthorisedAgentId,

    pub last_name: String,
    pub last_name_prefix: Option<String>,
    pub initials: String,

    pub locality: String,
    pub postal_code: String,
    pub house_number: String,
    pub house_number_addition: Option<String>,
    pub street_name: String,

    #[allow(unused)]
    pub created_at: DateTime<Utc>,
    #[allow(unused)]
    pub updated_at: DateTime<Utc>,
}

impl AuthorisedAgent {
    pub fn last_name_with_prefix(&self) -> String {
        if let Some(prefix) = &self.last_name_prefix {
            format!("{prefix} {}", self.last_name)
        } else {
            self.last_name.clone()
        }
    }
}
