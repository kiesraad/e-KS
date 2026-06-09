use crate::{
    AppError, AppEvent, AppStore,
    common::{
        FullName, InfoProblems, LegalName, PotentialProblems, Problematic, Problems, Severity,
    },
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
    fn get_problems(&self, _: ()) -> Problems {
        Problems::merge(vec![
            self.name.get_problems(Severity::Warn),
            self.legal_name.get_problems(()),
        ])
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
