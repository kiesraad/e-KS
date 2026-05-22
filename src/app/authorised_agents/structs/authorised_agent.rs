use crate::{
    AppError, AppEvent, AppStore, OptionAsStrExt,
    common::{FullName, LegalName, PotentialProblems, Problematic, Severity},
    id_newtype,
};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct AuthorisedAgentId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AuthorisedAgent {
    pub id: AuthorisedAgentId,
    pub name: FullName,
    pub legal_name: Option<LegalName>,
}

impl Problematic for AuthorisedAgent {
    fn get_problems(&self) -> Vec<PotentialProblems> {
        let mut problems = self.name.potential_problems(Severity::Warn);
        if self.legal_name.is_empty_or_none() {
            problems.push(PotentialProblems::NoLegalName);
        }
        problems
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
