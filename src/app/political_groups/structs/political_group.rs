use crate::{
    AppError, AppEvent, AppStore, OptionAsStrExt,
    common::{DisplayName, LegalName, PreviousElectionResults},
    submit::{Completable, IncompleteItem},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalGroup {
    pub previous_election_results: Option<PreviousElectionResults>,
    pub legal_name: Option<LegalName>,
    pub display_name: Option<DisplayName>,
}

impl Completable for PoliticalGroup {
    fn incomplete_items(&self) -> Vec<IncompleteItem> {
        [
            self.legal_name.incomplete_items(),
            self.display_name.incomplete_items(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl PoliticalGroup {
    pub fn is_basic_info_empty(&self) -> bool {
        self.previous_election_results.is_none()
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn incomplete_items_empty() {
        let empty_items = PoliticalGroup {
            previous_election_results: None,
            legal_name: None,
            display_name: None,
        }
        .incomplete_items();

        assert_eq!(empty_items.len(), 2);
        assert!(empty_items.contains(&IncompleteItem::NoLegalName));
        assert!(empty_items.contains(&IncompleteItem::NoDisplayName));
    }

    #[test]
    fn incomplete_items_complete() {
        let complete_items = PoliticalGroup {
            previous_election_results: Some(PreviousElectionResults::OneToFifteenSeats),
            legal_name: LegalName::from_str("test").ok(),
            display_name: DisplayName::from_str("test").ok(),
        }
        .incomplete_items();

        assert!(complete_items.is_empty());
    }
}
