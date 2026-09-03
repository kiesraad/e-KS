use super::PaperCorrected;
use crate::{CsbStream, projection::WithCorrections, structs::list_submitters::ListSubmitter};

/// A (substitute) list submitter with its rows diffed against the corrections.
pub struct PaperCorrectedSubmitter {
    pub heading: String,
    pub initials: PaperCorrected,
    pub last_name: PaperCorrected,
    pub postal_code: PaperCorrected,
    pub house_number: PaperCorrected,
    pub street_name: PaperCorrected,
    pub locality: PaperCorrected,
    pub state_or_province: PaperCorrected,
    pub country: PaperCorrected,
    pub is_foreign: bool,
}

impl PaperCorrectedSubmitter {
    fn from_pair(imported: Option<&ListSubmitter>, corrected: Option<&ListSubmitter>) -> Self {
        Self {
            heading: corrected
                .or(imported)
                .map(|s| s.name.display())
                .unwrap_or_default(),
            initials: PaperCorrected::from_field(imported, corrected, |s| {
                s.name.initials.to_string()
            }),
            last_name: PaperCorrected::from_field(imported, corrected, |s| {
                s.name.last_name_with_prefix()
            }),
            postal_code: PaperCorrected::from_field(imported, corrected, |s| {
                s.address.postal_code().unwrap_or_default()
            }),
            house_number: PaperCorrected::from_field(imported, corrected, |s| {
                s.address.house_number().unwrap_or_default()
            }),
            street_name: PaperCorrected::from_field(imported, corrected, |s| {
                s.address.street_name().unwrap_or_default()
            }),
            locality: PaperCorrected::from_field(imported, corrected, |s| {
                s.address.locality().unwrap_or_default()
            }),
            state_or_province: PaperCorrected::from_field(imported, corrected, |s| {
                s.address.state_or_province().unwrap_or_default()
            }),
            country: PaperCorrected::from_field(imported, corrected, |s| {
                s.address.country().unwrap_or_default()
            }),
            is_foreign: Self::is_foreign(imported, corrected),
        }
    }
    /// foreign if:
    /// 1. Both imported and corrected aren't [None]
    /// 2. Both are not Dutch addresses
    ///
    /// In any other case, a submitter is not considered foreign
    fn is_foreign(imported: Option<&ListSubmitter>, corrected: Option<&ListSubmitter>) -> bool {
        let Some(imported) = imported else {
            return false;
        };
        let Some(corrected) = corrected else {
            return false;
        };
        !imported.address.is_dutch() || !corrected.address.is_dutch()
    }
}

/// The list submitter diffed against the corrections, or `None` when the
/// corrections have none (never present, or deleted by the corrections).
pub fn paper_corrected_list_submitter(store: &CsbStream) -> Option<PaperCorrectedSubmitter> {
    let imported = store.get_list_submitter(WithCorrections::None);
    let corrected = store.get_list_submitter(WithCorrections::Paper);

    if corrected.is_empty() {
        return None;
    }

    Some(PaperCorrectedSubmitter::from_pair(
        (!imported.is_empty()).then_some(&imported),
        Some(&corrected),
    ))
}

/// The substitute submitters paired with their corrected counterparts by id;
/// substitutes added by the corrections are appended, substitutes deleted by
/// the corrections are hidden.
pub fn paper_corrected_substitute_submitters(store: &CsbStream) -> Vec<PaperCorrectedSubmitter> {
    let imported = store.get_substitute_submitters(WithCorrections::None);
    let corrected = store.get_substitute_submitters(WithCorrections::Paper);

    let mut rows: Vec<PaperCorrectedSubmitter> = imported
        .iter()
        .filter_map(|submitter| {
            let counterpart = corrected.iter().find(|c| c.id == submitter.id)?;
            Some(PaperCorrectedSubmitter::from_pair(
                Some(submitter),
                Some(counterpart),
            ))
        })
        .collect();

    rows.extend(
        corrected
            .iter()
            .filter(|c| !imported.iter().any(|submitter| submitter.id == c.id))
            .map(|c| PaperCorrectedSubmitter::from_pair(None, Some(c))),
    );

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CsbStore,
        structs::{
            common::{
                Address::{self},
                DutchAddress, InternationalAddress,
            },
            list_submitters::ListSubmitterId,
        },
        test_utils::sample_list_submitter,
    };

    #[test]
    fn list_submitter_deleted_by_the_corrections_is_hidden() {
        let store = CsbStore::new_for_test();
        store.data.write().imported_data.list_submitter =
            sample_list_submitter(ListSubmitterId::new());

        assert!(paper_corrected_list_submitter(&store).is_none());
    }

    #[test]
    fn unchanged_list_submitter_is_shown() {
        let store = CsbStore::new_for_test();
        let submitter = sample_list_submitter(ListSubmitterId::new());
        {
            let mut data = store.data.write();
            data.imported_data.list_submitter = submitter.clone();
            data.paper_corrected_data.list_submitter = submitter;
        }

        let row = paper_corrected_list_submitter(&store).unwrap();
        assert!(!row.last_name.differs());
    }

    #[test]
    fn submitter_country_correction_is_shown() {
        use crate::structs::common::{Address, InternationalAddress};

        let store = CsbStore::new_for_test();
        let submitter = sample_list_submitter(ListSubmitterId::new());
        let mut corrected = submitter.clone();
        corrected.address = Address::International(InternationalAddress {
            country: Some("BE".parse().unwrap()),
            state_or_province: Some("Antwerpen".parse().unwrap()),
            ..Default::default()
        });
        {
            let mut data = store.data.write();
            data.imported_data.list_submitter = submitter;
            data.paper_corrected_data.list_submitter = corrected;
        }

        let row = paper_corrected_list_submitter(&store).unwrap();
        assert!(row.country.differs());
        assert_eq!(row.country.corrected, "BE");
        assert!(row.state_or_province.differs());
        assert_eq!(row.state_or_province.corrected, "Antwerpen");
    }

    #[test]
    fn substitute_submitter_deleted_by_the_corrections_is_hidden() {
        let store = CsbStore::new_for_test();
        let kept = sample_list_submitter(ListSubmitterId::new());
        let deleted = sample_list_submitter(ListSubmitterId::new());
        {
            let mut data = store.data.write();
            data.imported_data.substitute_submitters = vec![kept.clone(), deleted];
            data.paper_corrected_data.substitute_submitters = vec![kept];
        }

        let rows = paper_corrected_substitute_submitters(&store);
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn is_foreign() {
        let dutch = Address::Dutch(DutchAddress {
            ..Default::default()
        });
        let foreign = Address::International(InternationalAddress {
            ..Default::default()
        });

        let mut s1 = sample_list_submitter(ListSubmitterId::new());
        let mut s2 = sample_list_submitter(ListSubmitterId::new());
        s1.address = foreign.clone();
        s2.address = foreign.clone();

        assert!(PaperCorrectedSubmitter::is_foreign(Some(&s1), Some(&s2)));

        s1.address = dutch.clone();
        assert!(PaperCorrectedSubmitter::is_foreign(Some(&s1), Some(&s2)));
        assert!(PaperCorrectedSubmitter::is_foreign(Some(&s2), Some(&s1)));

        assert!(!PaperCorrectedSubmitter::is_foreign(None, Some(&s2)));
        assert!(!PaperCorrectedSubmitter::is_foreign(Some(&s2), None));

        s2.address = dutch.clone();
        assert!(!PaperCorrectedSubmitter::is_foreign(Some(&s1), Some(&s2)));
        assert!(!PaperCorrectedSubmitter::is_foreign(Some(&s2), Some(&s1)));

        assert!(!PaperCorrectedSubmitter::is_foreign(None, Some(&s2)));
        assert!(!PaperCorrectedSubmitter::is_foreign(Some(&s2), None));

        assert!(!PaperCorrectedSubmitter::is_foreign(None, None));
    }
}
