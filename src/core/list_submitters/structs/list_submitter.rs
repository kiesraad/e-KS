use crate::{AppError, AppEvent, AppStore, DutchAddress, FullName, UtcDateTime, id_newtype};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct ListSubmitterId);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ListSubmitter {
    pub id: ListSubmitterId,
    pub name: FullName,
    pub address: DutchAddress,

    #[allow(unused)]
    pub created_at: UtcDateTime,
    #[allow(unused)]
    pub updated_at: UtcDateTime,
}

impl ListSubmitter {
    pub fn is_complete(&self) -> bool {
        self.name.is_complete() && self.is_address_complete()
    }

    pub fn is_address_complete(&self) -> bool {
        self.address.is_complete()
    }

    pub fn last_name_with_prefix(&self) -> String {
        self.name.last_name_with_prefix()
    }

    pub async fn create(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::CreateListSubmitter(ListSubmitter {
                created_at: UtcDateTime::now(),
                updated_at: UtcDateTime::now(),
                ..self.clone()
            }))
            .await
    }

    pub async fn update(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateListSubmitter(self.clone()))
            .await
    }

    pub async fn delete_by_id(
        store: &AppStore,
        list_submitter_id: ListSubmitterId,
    ) -> Result<(), AppError> {
        store
            .update(AppEvent::DeleteListSubmitter(list_submitter_id))
            .await?;

        Ok(())
    }
}
