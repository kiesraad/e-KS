use crate::{
    AppError, AppEvent, AppStore, OptionAsStrExt,
    common::{DisplayName, PotentialProblems, Problematic},
    list_designation::ListDesignation,
};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalGroup {
    pub display_name: Option<DisplayName>,
    pub list_designation: ListDesignation,
}

impl Problematic for PoliticalGroup {
    fn get_problems(&self) -> Vec<PotentialProblems> {
        [
            // self.legal_name.get_problems(),
            self.display_name.get_problems(),
            // self.previous_election_results
            //     .is_none()
            //     .then_some(vec![PotentialProblems::NoPreviousElectionResults])
            // .unwrap_or_default(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl PoliticalGroup {
    pub fn is_basic_info_empty(&self) -> bool {
        // self.previous_election_results.is_none()
        //     && self.legal_name.is_empty_or_none() &&
        self.display_name.is_empty_or_none()
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
    use crate::list_designation::PoliticalEntity;

    use super::*;

    impl PoliticalGroup {
        pub fn unwrap_standalone(&self) -> &PoliticalEntity {
            match &self.list_designation {
                ListDesignation::Standalone(entity) => entity,
                _ => panic!("Not a standalone list!"),
            }
        }

        pub fn unwrap_standalone_mut(&mut self) -> &mut PoliticalEntity {
            match &mut self.list_designation {
                ListDesignation::Standalone(entity) => entity,
                _ => panic!("Not a standalone list!"),
            }
        }
    }

    // use std::str::FromStr;

    // #[test]
    // fn incomplete_items_empty() {
    //     let empty_items = PoliticalGroup {
    //         previous_election_results: None,
    //         legal_name: None,
    //         display_name: None,
    //     }
    //     .get_problems();

    //     assert_eq!(empty_items.len(), 3);
    //     assert!(empty_items.contains(&PotentialProblems::NoLegalName));
    //     assert!(empty_items.contains(&PotentialProblems::NoDisplayName));
    //     assert!(empty_items.contains(&PotentialProblems::NoPreviousElectionResults));
    // }

    // #[test]
    // fn incomplete_items_complete() {
    //     let complete_items = PoliticalGroup {
    //         previous_election_results: Some(PreviousElectionResults::OneToFifteenSeats),
    //         legal_name: LegalName::from_str("test").ok(),
    //         display_name: DisplayName::from_str("test").ok(),
    //     }
    //     .get_problems();

    //     assert!(complete_items.is_empty());
    // }
}
