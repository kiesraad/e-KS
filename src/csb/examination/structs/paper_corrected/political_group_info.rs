use super::PaperCorrected;
use crate::{
    CsbStream, Locale,
    projection::WithCorrections,
    structs::{
        common::PreviousElectionResults, list_designation::ListDesignation,
        political_groups::PoliticalGroup,
    },
    trans,
};

/// The political group rows of the general information page.
pub struct PaperCorrectedPoliticalGroupInfo {
    pub appellation: PaperCorrected,
    pub list_type: PaperCorrected,
    pub previous_results: PaperCorrected,
}

impl PaperCorrectedPoliticalGroupInfo {
    pub fn new(store: &CsbStream, locale: Locale) -> Self {
        let imported_group = store.get_political_group(WithCorrections::None);
        let paper_corrected_group = store.get_political_group(WithCorrections::Paper);

        Self {
            appellation: PaperCorrected::new(
                store.get_appellation(WithCorrections::None),
                store.get_appellation(WithCorrections::Paper),
            )
            .with_csb_correction(Some(store.get_appellation(WithCorrections::All))),
            list_type: PaperCorrected::new(
                list_type_label(&imported_group, locale),
                list_type_label(&paper_corrected_group, locale),
            ),
            previous_results: PaperCorrected::new(
                previous_results_label(&imported_group, locale),
                previous_results_label(&paper_corrected_group, locale),
            ),
        }
    }
}

fn list_type_label(political_group: &PoliticalGroup, locale: Locale) -> String {
    match political_group.list_designation {
        Some(ListDesignation::Standalone) => trans!("political_group.type.registered_name", locale),
        Some(ListDesignation::Combined) => trans!("political_group.type.name_combination", locale),
        Some(ListDesignation::Blank) => trans!("political_group.type.blank_name", locale),
        None => "-".to_string(),
    }
}

fn previous_results_label(political_group: &PoliticalGroup, locale: Locale) -> String {
    match political_group.previous_election_results {
        Some(PreviousElectionResults::ZeroSeats) => {
            trans!("political_group.type.zero_seats", locale)
        }
        Some(PreviousElectionResults::OneToFifteenSeats) => {
            trans!("political_group.type.one_to_fifteen_seats", locale)
        }
        Some(PreviousElectionResults::SixteenOrMoreSeats) => {
            trans!("political_group.type.sixteen_or_more_seats", locale)
        }
        None => "-".to_string(),
    }
}
