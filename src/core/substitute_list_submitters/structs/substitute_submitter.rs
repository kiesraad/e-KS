use crate::{AppError, AppEvent, AppStore, DutchAddress, FullName, UtcDateTime, id_newtype};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct SubstituteSubmitterId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SubstituteSubmitter {
    pub id: SubstituteSubmitterId,
    pub name: FullName,
    pub address: DutchAddress,

    #[allow(unused)]
    pub created_at: UtcDateTime,
    #[allow(unused)]
    pub updated_at: UtcDateTime,
}

impl SubstituteSubmitter {
    pub fn is_complete(&self) -> bool {
        self.name.is_complete() && self.address.is_complete()
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
            .update(AppEvent::DeleteSubstituteSubmitter {
                substitute_submitter_id: self.id,
                updated_at: UtcDateTime::now(),
            })
            .await
    }
}
