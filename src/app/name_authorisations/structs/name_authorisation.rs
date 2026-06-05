use crate::{
    AppError, AppEvent, AppStore,
    common::{FullName, LegalName, PotentialProblems, Problematic, Severity},
    id_newtype,
};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct NameAuthorisationId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NameAuthorisation {
    pub id: NameAuthorisationId,
    pub name: FullName,
    pub legal_name: LegalName,
}

impl Problematic<()> for NameAuthorisation {
    fn get_problems(&self, _: ()) -> Vec<PotentialProblems> {
        let mut problems = self.name.get_problems(Severity::Warn);
        if self.legal_name.to_string().is_empty() {
            problems.push(PotentialProblems::NoLegalName);
        }
        problems
    }
}

impl NameAuthorisation {
    pub async fn create(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::CreateNameAuthorisation(self.clone()))
            .await
    }

    pub async fn update(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateNameAuthorisation(self.clone()))
            .await
    }

    pub async fn delete(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::DeleteNameAuthorisation(self.id))
            .await
    }
}
