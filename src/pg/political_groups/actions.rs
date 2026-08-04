//! Store-backed operations for [`PoliticalGroup`].

use crate::{
    AppError, PgEvent, PgStore, QueryParamState,
    structs::{list_designation::ListDesignation, political_groups::PoliticalGroup},
};
use axum_extra::routing::TypedPath;

impl PoliticalGroup {
    /// Check if the full general information section is empty
    pub fn is_general_information_empty(&self, store: &PgStore) -> bool {
        self.is_list_designation_type_empty()
            && self.is_group_information_empty()
            && store.get_name_authorisations().is_empty()
            && store.get_list_submitter().is_empty()
            && store.get_substitute_submitters().is_empty()
    }

    /// URL for the "General information" step.
    /// Includes `initial=true` when all fields are still empty, so the
    /// first-visit flow suppresses warnings for steps not yet reached.
    pub fn general_information_path(&self, store: &PgStore) -> String {
        if self.is_general_information_empty(store) {
            ListDesignation::update_path()
                .with_query_params(QueryParamState::initial())
                .to_string()
        } else {
            ListDesignation::update_path().to_string()
        }
    }

    pub async fn create(&self, store: &PgStore) -> Result<(), AppError> {
        store
            .update(PgEvent::UpdatePoliticalGroup(self.clone()))
            .await
    }

    pub async fn update(&self, store: &PgStore) -> Result<(), AppError> {
        store
            .update(PgEvent::UpdatePoliticalGroup(self.clone()))
            .await
    }
}
