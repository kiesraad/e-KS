use crate::{
    AppError, AppEvent, AppStore, OptionAsStrExt,
    common::{
        DisplayName, InfoProblems, PotentialProblems, PreviousElectionResults, Problematic,
        Problems,
    },
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
            self.previous_election_results.get_problems(()),
        ])
    }
}

impl PoliticalGroup {
    pub fn effective_display_name(&self) -> Result<String, AppError> {
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

    pub fn is_basic_info_empty(&self) -> bool {
        self.display_name.is_empty_or_none() && self.previous_election_results.is_none()
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
        assert!(empty_items.potential_problems.contains(&PotentialProblems::NoDisplayName));
        
        assert_eq!(empty_items.info_problems.len(), 2);
        assert!(empty_items.info_problems.contains(&InfoProblems::NoPreviousElectionResults));
        assert!(empty_items.info_problems.contains(&InfoProblems::NoListDesignation));
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
        assert_eq!(group.effective_display_name().unwrap(), "test");
        assert_eq!(group.get_max_candidates(), 80);
        assert!(group.was_previously_seated());

        // the set values should be ignored when switching to a blank list
        group.list_designation = Some(ListDesignation::Blank);
        assert_eq!(group.effective_display_name().unwrap(), "");
        assert_eq!(group.get_max_candidates(), 50);
        assert!(!group.was_previously_seated());
    }
}
