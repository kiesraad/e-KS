use crate::{
    AppError, AppEvent, Store,
    common::{DutchAddress, FullName},
    id_newtype,
};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct ListSubmitterId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ListSubmitter {
    pub id: ListSubmitterId,
    pub name: FullName,
    pub address: DutchAddress,
}

impl ListSubmitter {
    pub fn is_complete(&self) -> bool {
        self.name.is_complete() && self.address.is_complete()
    }

    pub async fn create(&self, store: &Store) -> Result<(), AppError> {
        store
            .update(AppEvent::CreateListSubmitter(self.clone()))
            .await
    }

    pub async fn update(&self, store: &Store) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateListSubmitter(self.clone()))
            .await
    }

    pub async fn delete(&self, store: &Store) -> Result<(), AppError> {
        store
            .update(AppEvent::DeleteListSubmitter {
                list_submitter_id: self.id,
            })
            .await
    }
}
