use crate::{AppError, AppEvent, AppStore, id_newtype};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct SubstituteSubmitterId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
    pub fn is_complete(&self) -> bool {
        !self.initials.is_empty() && !self.last_name.is_empty() && self.is_address_complete()
    }

    pub fn is_empty(&self) -> bool {
        self.initials.is_empty()
            && self.last_name.is_empty()
            && self.last_name_prefix.as_deref().unwrap_or("").is_empty()
            && self.locality.as_deref().unwrap_or("").is_empty()
            && self.postal_code.as_deref().unwrap_or("").is_empty()
            && self.house_number.as_deref().unwrap_or("").is_empty()
            && self.house_number_addition.as_deref().unwrap_or("").is_empty()
            && self.street_name.as_deref().unwrap_or("").is_empty()
    }

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

    pub async fn create(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::CreateSubstituteSubmitter(self.clone()))
            .await
    }

    pub async fn update(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateSubstituteSubmitter(self.clone()))
            .await
    }

    pub async fn delete(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::DeleteSubstituteSubmitter(self.id))
            .await
    }

    pub async fn delete_by_id(
        store: &AppStore,
        substitute_submitter_id: SubstituteSubmitterId,
    ) -> Result<(), AppError> {
        store
            .update(AppEvent::DeleteSubstituteSubmitter(substitute_submitter_id))
            .await
    }
}
