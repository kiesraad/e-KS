use crate::{AppError, AppEvent, Store, common::FullName, id_newtype};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct AuthorisedAgentId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AuthorisedAgent {
    pub id: AuthorisedAgentId,
    pub name: FullName,
}

impl AuthorisedAgent {
    pub fn is_complete(&self) -> bool {
        self.name.is_complete()
    }

    pub async fn create(&self, store: &Store) -> Result<(), AppError> {
        store
            .update(AppEvent::CreateAuthorisedAgent(self.clone()))
            .await
    }

    pub async fn update(&self, store: &Store) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateAuthorisedAgent(self.clone()))
            .await
    }

    pub async fn delete(&self, store: &Store) -> Result<(), AppError> {
        store.update(AppEvent::DeleteAuthorisedAgent(self.id)).await
    }
}
