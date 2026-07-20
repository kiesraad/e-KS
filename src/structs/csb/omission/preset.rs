use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

/// A predefined omission ("verzuim") offered as a quick-fill suggestion in the
/// add-omission dialog, split into the model I 1 [`Self::description`] and the
/// [`Self::help_text`] telling the political group how to restore it.
#[derive(Debug, Clone, Deserialize)]
pub struct PresetOmission {
    /// Short title shown in the quick-fill pill.
    pub title: String,
    pub description: String,
    pub help_text: String,
    #[serde(default = "super::recoverable_by_default")]
    pub recoverable: bool,
}

/// Values used to interpolate the `{token}` placeholders in an omission
/// description with the correct data for the referenced item.
#[derive(Debug, Default, Clone)]
pub struct OmissionPlaceholders {
    /// `{candidate_number}`: the candidate's position on the list.
    pub candidate_number: Option<String>,
    /// `{candidate_name}`: the candidate's initials and last name.
    pub candidate_name: Option<String>,
}

impl OmissionPlaceholders {
    /// Replace every placeholder we have a value for, leaving the rest in place
    /// (as `{token}`) for the committee to fill in.
    pub fn interpolate(&self, template: &str) -> String {
        let mut result = template.to_string();
        for (token, value) in [
            ("{candidate_number}", &self.candidate_number),
            ("{candidate_name}", &self.candidate_name),
        ] {
            if let Some(value) = value {
                result = result.replace(token, value);
            }
        }
        result
    }
}

/// The standard omissions per omission type, loaded from `omissions.json`.
pub static PRESET_OMISSIONS: LazyLock<HashMap<String, Vec<PresetOmission>>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("omissions.json")).expect("omissions.json should be valid")
});

impl super::OmissionType {
    /// The predefined omissions offered as quick-fill suggestions for this type.
    pub fn presets(self) -> &'static [PresetOmission] {
        PRESET_OMISSIONS
            .get(self.as_str())
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::structs::csb::omission::OmissionType;

    #[test]
    fn presets_are_loaded_from_json_per_type() {
        assert_eq!(OmissionType::PoliticalGroup.presets().len(), 6);
        assert_eq!(OmissionType::CandidateList.presets().len(), 4);
        assert_eq!(OmissionType::Candidate.presets().len(), 12);

        // Every preset carries a title and description; irreparable defects have
        // no help text.
        assert!(
            OmissionType::PoliticalGroup
                .presets()
                .iter()
                .all(|p| !p.title.is_empty() && !p.description.is_empty())
        );
        assert!(
            OmissionType::PoliticalGroup
                .presets()
                .iter()
                .any(|p| p.help_text.is_empty())
        );
    }

    #[test]
    fn presets_carry_the_recoverable_flag() {
        // Most omissions are recoverable ("herstelbaar").
        assert!(
            OmissionType::Candidate
                .presets()
                .iter()
                .any(|p| p.recoverable)
        );
        // Irreparable defects ("onherstelbaar verzuim") have no help text and are
        // flagged as non-recoverable.
        assert!(
            OmissionType::PoliticalGroup
                .presets()
                .iter()
                .any(|p| !p.recoverable)
        );
        assert!(
            OmissionType::PoliticalGroup
                .presets()
                .iter()
                .all(|p| p.recoverable || p.help_text.is_empty())
        );
    }

    #[test]
    fn interpolate_fills_known_tokens_and_keeps_the_rest() {
        let placeholders = OmissionPlaceholders {
            candidate_number: Some("3".to_string()),
            candidate_name: Some("A.B. de Vries".to_string()),
        };

        let result = placeholders
            .interpolate("Kandidaat nr. {candidate_number}, {candidate_name} ... {designation}");

        assert_eq!(
            result,
            // The known tokens are filled; the manual one is left in place.
            "Kandidaat nr. 3, A.B. de Vries ... {designation}"
        );
    }

    #[test]
    fn interpolate_leaves_all_tokens_without_values() {
        let result =
            OmissionPlaceholders::default().interpolate("nr. {candidate_number} {candidate_name}");

        assert_eq!(result, "nr. {candidate_number} {candidate_name}");
    }
}
