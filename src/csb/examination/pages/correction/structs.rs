use crate::{
    CsbStore,
    csb::examination::{pages::correction::extract_field, structs::CandidateCorrectionField},
    persons::PersonId,
};

/// Which type of input to render in the correction overlay.
pub enum CorrectionFieldType {
    Text,
    Initials,
    DateOfBirth,
    PlaceOfResidence,
}

impl From<CandidateCorrectionField> for CorrectionFieldType {
    fn from(field: CandidateCorrectionField) -> Self {
        match field {
            CandidateCorrectionField::Initials => Self::Initials,
            CandidateCorrectionField::LastName => Self::Text,
            CandidateCorrectionField::DateOfBirth => Self::DateOfBirth,
            CandidateCorrectionField::PlaceOfResidence => Self::PlaceOfResidence,
        }
    }
}

/// Display arguments for the correction overlay, grouped to keep
/// `render_correction` within the argument-count limit.
pub(crate) struct CorrectionDisplay {
    pub(crate) label: String,
    pub(crate) imported_value: String,
    pub(crate) paper_corrected_value: Option<String>,
    pub(crate) field_type: CorrectionFieldType,
}

/// The three value strings needed for the correction overlay, extracted from
/// the three data projections.
pub(crate) struct FieldValues {
    imported: String,
    paper_corrected: Option<String>,
    current_correction: Option<String>,
}

impl FieldValues {
    pub(crate) fn for_display_name(store: &CsbStore) -> Self {
        let imported = store
            .get_imported_political_group()
            .display_name
            .as_ref()
            .map(|d| d.to_string())
            .unwrap_or_default();
        let paper_corrected = store
            .paper_corrected()
            .get_political_group()
            .display_name
            .as_ref()
            .map(|d| d.to_string())
            .filter(|d| d != &imported);
        let current_correction = store
            .get_csb_corrected_display_name()
            .map(|d| d.to_string());
        Self {
            imported,
            paper_corrected,
            current_correction,
        }
    }

    pub(crate) fn for_person(
        store: &CsbStore,
        person_id: PersonId,
        field: CandidateCorrectionField,
    ) -> Self {
        let imported = store.get_imported_person(person_id);
        let paper_corrected = store.paper_corrected().get_person(person_id).ok();
        let csb_corrected = store.get_csb_corrected_person(person_id);

        let imported = imported
            .as_ref()
            .map(|p| extract_field(field, p))
            .unwrap_or_default();
        let paper_corrected = paper_corrected
            .as_ref()
            .map(|p| extract_field(field, p))
            .filter(|v| v != &imported)
            .filter(|v| !v.is_empty());
        let current_correction = csb_corrected
            .as_ref()
            .map(|p| extract_field(field, p))
            .filter(|v| !v.is_empty());

        Self {
            imported,
            paper_corrected,
            current_correction,
        }
    }

    /// The value to pre-fill the form with: CSB correction if one
    /// exists, otherwise paper-corrected, otherwise imported.
    pub(crate) fn prefill(&self) -> String {
        self.current_correction
            .clone()
            .or_else(|| self.paper_corrected.clone())
            .unwrap_or_else(|| self.imported.clone())
    }

    pub(crate) fn into_display(
        self,
        label: String,
        field_type: CorrectionFieldType,
    ) -> CorrectionDisplay {
        CorrectionDisplay {
            label,
            imported_value: self.imported,
            paper_corrected_value: self.paper_corrected,
            field_type,
        }
    }

    pub(crate) fn into_person_display(
        self,
        field: CandidateCorrectionField,
        locale: crate::Locale,
    ) -> CorrectionDisplay {
        self.into_display(field.label(locale), field.into())
    }
}
