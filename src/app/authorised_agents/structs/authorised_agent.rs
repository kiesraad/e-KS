use crate::{
    AppError, AppEvent, AppStore,
    common::FullName,
    id_newtype,
    submit::{Completable, IncompleteItem, Severity},
};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct AuthorisedAgentId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AuthorisedAgent {
    pub id: AuthorisedAgentId,
    pub name: FullName,
}

impl Completable for AuthorisedAgent {
    fn incomplete_items(&self) -> Vec<IncompleteItem> {
        self.name.completable_items(Severity::Warn)
    }
}

impl AuthorisedAgent {
    pub fn is_complete(&self) -> bool {
        self.name.is_complete()
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
