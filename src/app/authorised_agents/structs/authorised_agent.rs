use crate::{
    AppError, AppEvent, AppStore,
    common::FullName,
    id_newtype,
    submit::{PotentialProblems, Problematic, Severity},
};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct AuthorisedAgentId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AuthorisedAgent {
    pub id: AuthorisedAgentId,
    pub name: FullName,
}

impl Problematic for AuthorisedAgent {
    fn get_problems(&self) -> Vec<PotentialProblems> {
        self.name.potential_problems(Severity::Warn)
    }
}

impl AuthorisedAgent {
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
