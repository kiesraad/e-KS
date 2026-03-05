use crate::{
    AppError, AppEvent, AppStore, OptionAsStrExt, PoliticalGroupId,
    common::{DisplayName, LegalName},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalGroup {
    pub id: PoliticalGroupId,

    pub long_list_allowed: Option<bool>,
    pub legal_name: Option<LegalName>,
    pub display_name: Option<DisplayName>,
}

impl PoliticalGroup {
    pub fn is_basic_info_complete(&self) -> bool {
        self.long_list_allowed.is_some()
            && !self.legal_name.is_empty_or_none()
            && !self.display_name.is_empty_or_none()
    }

    pub fn is_basic_info_empty(&self) -> bool {
        self.long_list_allowed.is_none()
            && self.legal_name.is_empty_or_none()
            && self.display_name.is_empty_or_none()
    }

    pub async fn create(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdatePoliticalGroup(self.clone()))
            .await
    }

    pub async fn update(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdatePoliticalGroup(self.clone()))
            .await
    }
}
