use crate::{AppError, AppEvent, AppStore, FullName, UtcDateTime, id_newtype};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct AuthorisedAgentId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AuthorisedAgent {
    pub id: AuthorisedAgentId,
    pub name: FullName,

    #[allow(unused)]
    pub created_at: UtcDateTime,
    #[allow(unused)]
    pub updated_at: UtcDateTime,
}

impl AuthorisedAgent {
    pub fn is_complete(&self) -> bool {
        self.name.is_complete()
    }

    pub fn last_name_with_prefix(&self) -> String {
        self.name.last_name_with_prefix()
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
}
