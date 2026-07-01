use crate::{
    AppError, AppEvent, AppStore,
    common::{FullName, LegalName, PotentialProblems, Problematic, Problems, Severity},
    id_newtype,
    list_designation::ListDesignation,
};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct NameAuthorisationId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
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
    pub fn get_size_problems(
        list_designation: Option<ListDesignation>,
        name_authorisation_count: usize,
    ) -> Option<PotentialProblems> {
        match list_designation.unwrap_or(ListDesignation::Standalone) {
            ListDesignation::Standalone if name_authorisation_count > 1 => {
                Some(PotentialProblems::TooManyAuthorizedNames {
                    count: name_authorisation_count - 1,
                })
            }
            ListDesignation::Standalone if name_authorisation_count < 1 => {
                Some(PotentialProblems::TooFewAuthorizedNames { count: 1 })
            }
            ListDesignation::Combined if name_authorisation_count < 2 => {
                Some(PotentialProblems::TooFewAuthorizedNames {
                    count: 2 - name_authorisation_count,
                })
            }
            _ => None,
        }
    }

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
