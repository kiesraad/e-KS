use crate::{
    AppError, AppEvent, AppStore, OptionAsStrExt,
    common::{DisplayName, PotentialProblems, PreviousElectionResults, Problematic},
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
    fn get_problems(&self, _: ()) -> Vec<PotentialProblems> {
        [
            self.display_name.get_problems(()),
            self.previous_election_results
                .is_none()
                .then_some(vec![PotentialProblems::NoPreviousElectionResults])
                .unwrap_or_default(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl PoliticalGroup {
    pub fn get_max_candidates(&self) -> usize {
        match self.previous_election_results {
            Some(PreviousElectionResults::SixteenOrMoreSeats) => 80,
            _ => 50,
        }
    }

    pub fn was_previously_seated(&self) -> bool {
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

        assert_eq!(empty_items.len(), 2);
        assert!(empty_items.contains(&PotentialProblems::NoDisplayName));
        assert!(empty_items.contains(&PotentialProblems::NoPreviousElectionResults));
    }

    #[test]
    fn incomplete_items_complete() {
        let complete_items = PoliticalGroup {
            previous_election_results: Some(PreviousElectionResults::OneToFifteenSeats),
            list_designation: Some(ListDesignation::Standalone),
            display_name: DisplayName::from_str("test").ok(),
        }
        .get_problems(());

        assert!(complete_items.is_empty());
    }
}
