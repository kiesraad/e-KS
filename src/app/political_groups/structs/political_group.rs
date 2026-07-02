use axum_extra::routing::TypedPath;

use crate::{
    AppError, AppEvent, AppStore, OptionAsStrExt, QueryParamState,
    common::{DisplayName, PreviousElectionResults, Problematic, Problems},
    list_designation::ListDesignation,
};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalGroup {
    pub display_name: Option<DisplayName>,
    pub list_designation: Option<ListDesignation>,
    pub previous_election_results: Option<PreviousElectionResults>,
}

impl Problematic<()> for PoliticalGroup {
    fn get_problems(&self, _: ()) -> Problems {
        Problems::merge(vec![
            self.display_name.get_problems(self.list_designation),
            self.list_designation.get_problems(()),
            self.previous_election_results
                .get_problems(self.list_designation),
        ])
    }
}

impl PoliticalGroup {
    /// display name to use in exported documents such as EMLs or H-models
    pub fn display_name_for_exports(&self) -> Result<String, AppError> {
        if self.list_designation == Some(ListDesignation::Blank) {
            return Ok(String::new());
        }
        self.display_name
            .as_ref()
            .map(|d| Ok(d.to_string()))
            .unwrap_or(Err(AppError::IncompleteData(
                "Missing registered designation",
            )))
    }

    /// display name to use in the UI of the CSB module.
    /// TODO: figure out what to do with blanco lijsten, see #870
    pub fn csb_display_name(&self) -> String {
        self.display_name
            .as_deref()
            .unwrap_or(&"?".to_string())
            .clone()
    }

    pub fn get_max_candidates(&self) -> usize {
        if self.list_designation == Some(ListDesignation::Blank) {
            return 50;
        }
        match self.previous_election_results {
            Some(PreviousElectionResults::SixteenOrMoreSeats) => 80,
            _ => 50,
        }
    }

    pub fn was_previously_seated(&self) -> bool {
        if self.list_designation == Some(ListDesignation::Blank) {
            return false;
        }
        self.previous_election_results
            .is_some_and(|r| r != PreviousElectionResults::ZeroSeats)
    }

    pub fn is_list_designation_type_empty(&self) -> bool {
        self.list_designation.is_none()
    }

    pub fn is_group_information_empty(&self) -> bool {
        self.display_name.is_empty_or_none() && self.previous_election_results.is_none()
    }

    /// Check if the full general information section is empty
    pub fn is_general_information_empty(&self, store: &AppStore) -> bool {
        self.is_list_designation_type_empty()
            && self.is_group_information_empty()
            && store.get_name_authorisations().is_empty()
            && store.get_list_submitter().is_empty()
            && store.get_substitute_submitters().is_empty()
    }

    /// URL for the "General information" step.
    /// Includes `initial=true` when all fields are still empty, so the
    /// first-visit flow suppresses warnings for steps not yet reached.
    pub fn general_information_path(&self, store: &AppStore) -> String {
        if self.is_general_information_empty(store) {
            ListDesignation::update_path()
                .with_query_params(QueryParamState::initial())
                .to_string()
        } else {
            ListDesignation::update_path().to_string()
        }
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

#[cfg(test)]
mod tests {
    use crate::common::{InfoProblems, PotentialProblems};

    use super::*;

    use std::str::FromStr;

    #[test]
    fn incomplete_items_empty() {
        let empty_items = PoliticalGroup {
            previous_election_results: None,
            list_designation: None,
            display_name: None,
        }
        .get_problems(());

        assert_eq!(empty_items.potential_problems.len(), 1);
        assert!(
            empty_items
                .potential_problems
                .contains(&PotentialProblems::NoDisplayName)
        );

        assert_eq!(empty_items.info_problems.len(), 2);
        assert!(
            empty_items
                .info_problems
                .contains(&InfoProblems::NoPreviousElectionResults)
        );
        assert!(
            empty_items
                .info_problems
                .contains(&InfoProblems::NoListDesignation)
        );
    }

    #[test]
    fn complete_no_problems() {
        let problems = PoliticalGroup {
            previous_election_results: Some(PreviousElectionResults::OneToFifteenSeats),
            list_designation: Some(ListDesignation::Standalone),
            display_name: DisplayName::from_str("test").ok(),
        }
        .get_problems(());

        assert!(problems.potential_problems.is_empty());
        assert!(problems.info_problems.is_empty());
    }

    #[test]
    fn complete_blank_list_no_problems() {
        let problems = PoliticalGroup {
            previous_election_results: None,
            list_designation: Some(ListDesignation::Blank),
            display_name: None,
        }
        .get_problems(());
        assert!(problems.potential_problems.is_empty());
        assert!(problems.info_problems.is_empty());
    }

    #[test]
    fn blank_lists_force_defaults_even_if_set_differently() {
        let mut group = PoliticalGroup {
            previous_election_results: Some(PreviousElectionResults::SixteenOrMoreSeats),
            list_designation: Some(ListDesignation::Standalone),
            display_name: DisplayName::from_str("test").ok(),
        };
        assert_eq!(group.display_name_for_exports().unwrap(), "test");
        assert_eq!(group.get_max_candidates(), 80);
        assert!(group.was_previously_seated());

        // the set values should be ignored when switching to a blank list
        group.list_designation = Some(ListDesignation::Blank);
        assert_eq!(group.display_name_for_exports().unwrap(), "");
        assert_eq!(group.get_max_candidates(), 50);
        assert!(!group.was_previously_seated());
    }
}
