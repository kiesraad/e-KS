use crate::{AppError, AppEvent, AppStore, id_newtype};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct AuthorisedAgentId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AuthorisedAgent {
    pub id: AuthorisedAgentId,

    pub last_name: String,
    pub last_name_prefix: Option<String>,
    pub initials: String,

    #[allow(unused)]
    pub created_at: DateTime<Utc>,
    #[allow(unused)]
    pub updated_at: DateTime<Utc>,
}

impl AuthorisedAgent {
    pub fn is_complete(&self) -> bool {
        !self.initials.is_empty() && !self.last_name.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.initials.is_empty()
            && self.last_name.is_empty()
            && self.last_name_prefix.as_deref().unwrap_or("").is_empty()
    }

    pub fn last_name_with_prefix(&self) -> String {
        if let Some(prefix) = &self.last_name_prefix {
            format!("{prefix} {}", self.last_name)
        } else {
            self.last_name.clone()
        }
    }

    pub async fn create(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::CreateAuthorisedAgent(self.clone()))
            .await
    }

    pub async fn update(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateAuthorisedAgent(self.clone()))
            .await
    }

    pub async fn delete(&self, store: &AppStore) -> Result<(), AppError> {
        store.update(AppEvent::DeleteAuthorisedAgent(self.id)).await
    }

    pub async fn delete_by_id(
        store: &AppStore,
        authorised_agent_id: AuthorisedAgentId,
    ) -> Result<(), AppError> {
        store
            .update(AppEvent::DeleteAuthorisedAgent(authorised_agent_id))
            .await?;

        Ok(())
    }
}
