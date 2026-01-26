use crate::id_newtype;
use chrono::{DateTime, Utc};

id_newtype!(pub struct AuthorisedAgentId);

#[derive(Debug)]
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

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
