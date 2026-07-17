use std::{collections::HashMap, str::FromStr, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::{
    AnyLocale, AppError, CsbEvent, CsbStore, ElectionConfig, ElectoralDistrict,
    candidate_lists::{CandidateList, CandidateListId},
    common::UtcDateTime,
    form::ValidationError,
    id_newtype,
    persons::PersonId,
};

id_newtype!(pub struct OmissionId);

/// A predefined omission ("verzuim") offered as a quick-fill suggestion in the
/// add-omission dialog, split into the model I 1 [`Self::description`] and the
/// [`Self::help_text`] telling the political group how to restore it.
#[derive(Debug, Clone, Deserialize)]
pub struct PresetOmission {
    /// Short title shown in the quick-fill pill.
    pub title: String,
    pub description: String,
    pub help_text: String,
    #[serde(default = "recoverable_by_default")]
    pub recoverable: bool,
}

fn recoverable_by_default() -> bool {
    true
}

/// Values used to interpolate the `{token}` placeholders in an omission
/// description with the correct data for the referenced item.
#[derive(Debug, Default, Clone)]
pub struct OmissionPlaceholders {
    /// `{candidate_number}`: the candidate's position on the list.
    pub candidate_number: Option<String>,
    /// `{candidate_name}`: the candidate's initials and last name.
    pub candidate_name: Option<String>,
    /// `{districts}`: the electoral districts a candidate list was submitted for.
    pub districts: Option<String>,
}

impl OmissionPlaceholders {
    /// Replace every placeholder we have a value for, leaving the rest in place
    /// (as `{token}`) for the committee to fill in.
    pub fn interpolate(&self, template: &str) -> String {
        let mut result = template.to_string();
        for (token, value) in [
            ("{candidate_number}", &self.candidate_number),
            ("{candidate_name}", &self.candidate_name),
            ("{districts}", &self.districts),
        ] {
            if let Some(value) = value {
                result = result.replace(token, value);
            }
        }
        result
    }
}

/// The standard omissions per omission type, loaded from `omissions.json`.
static PRESET_OMISSIONS: LazyLock<HashMap<String, Vec<PresetOmission>>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("omissions.json")).expect("omissions.json should be valid")
});

/// The kind of item an omission is added to, carried as a path parameter so a
/// single "add omission" dialog can serve political groups, candidate lists and
/// candidates. Maps to a concrete [`OmissionCategory`] together with a
/// referenced item id (see [`OmissionCategory::from_type_and_reference`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmissionType {
    PoliticalGroup,
    CandidateList,
    Candidate,
}

impl OmissionType {
    fn as_str(self) -> &'static str {
        match self {
            OmissionType::PoliticalGroup => "political-group",
            OmissionType::CandidateList => "candidate-list",
            OmissionType::Candidate => "candidate",
        }
    }

    /// The predefined omissions offered as quick-fill suggestions for this type.
    ///
    /// A `general` candidate omission applies to the person on every list and
    /// draws from a separate set (`person`) than one scoped to the candidate on
    /// a specific list (`candidate`). `general` is meaningless for the other
    /// types, which only have a single set.
    pub fn presets(self, general: bool) -> &'static [PresetOmission] {
        let key = match self {
            OmissionType::Candidate if general => "person",
            _ => self.as_str(),
        };

        PRESET_OMISSIONS.get(key).map(Vec::as_slice).unwrap_or(&[])
    }
}

impl FromStr for OmissionType {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "political-group" => Ok(OmissionType::PoliticalGroup),
            "candidate-list" => Ok(OmissionType::CandidateList),
            "candidate" => Ok(OmissionType::Candidate),
            _ => Err(ValidationError::InvalidValue),
        }
    }
}

impl std::fmt::Display for OmissionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// Deserialize from a plain string so it works with the axum path deserializer
// (which drives every field through `deserialize_str`), mirroring `id_newtype`.
impl<'de> Deserialize<'de> for OmissionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Default, Debug, Serialize, Eq, PartialEq, Deserialize, Clone)]
pub enum OmissionCategory {
    /// E.g. missing deposit ("waarborgsom"), unidentified submitter,
    /// or problems with authorised agent and/or statutory name (H 3-1 / H 3-2)
    #[default]
    PoliticalGroup,
    /// E.g. missing or incorrect "ondersteuningsverklaringen" (H 4), which are per kieskring.
    /// Stores the specific electoral districts affected by the omission.
    CandidateList(Vec<ElectoralDistrict>),
    /// E.g. missing or invalid candidate data, missing or invalid "instemmingsverklaring" (H 9),
    /// missing copy of identity document
    Candidate {
        person: PersonId,
        /// The candidate list to which this applies, leave None if it applies to all candidate lists
        list: Option<CandidateListId>,
    },
}

const ALL_DISTRICTS: &str = "alle kieskringen";

impl OmissionCategory {
    /// Build the category for a newly added omission from the parameters of the
    /// "add omission" dialog for [`OmissionType::PoliticalGroup`] and
    /// [`OmissionType::Candidate`]. For candidate list omissions, construct the
    /// category directly with the selected districts.
    pub fn from_type_and_reference(
        omission_type: OmissionType,
        reference: uuid::Uuid,
        list: Option<CandidateListId>,
    ) -> Self {
        match omission_type {
            OmissionType::PoliticalGroup => OmissionCategory::PoliticalGroup,
            OmissionType::CandidateList => {
                unreachable!("CandidateList omissions must be created with explicit districts")
            }
            OmissionType::Candidate => OmissionCategory::Candidate {
                person: reference.into(),
                list,
            },
        }
    }

    /// Returns the electoral district string for use in model I 4.
    pub fn electoral_district(
        &self,
        store: &CsbStore,
        election: &ElectionConfig,
    ) -> Result<String, AppError> {
        match self {
            OmissionCategory::PoliticalGroup | OmissionCategory::Candidate { list: None, .. } => {
                Ok(ALL_DISTRICTS.to_string())
            }
            OmissionCategory::CandidateList(districts) => {
                Ok(format_districts_for_districts(districts, election))
            }
            OmissionCategory::Candidate { list: Some(id), .. } => store
                .get_candidate_list(*id)
                .as_ref()
                .map(|list| format_districts_for_list(list, election))
                .ok_or(AppError::GenericNotFound),
        }
    }
}

fn format_districts_for_list(list: &CandidateList, election: &ElectionConfig) -> String {
    format_districts_for_districts(&list.electoral_districts, election)
}

fn format_districts_for_districts(
    districts: &[ElectoralDistrict],
    election: &ElectionConfig,
) -> String {
    if election
        .electoral_districts()
        .iter()
        .all(|d| districts.contains(d))
    {
        ALL_DISTRICTS.to_string()
    } else {
        format_districts(districts)
    }
}

fn format_districts(districts: &[ElectoralDistrict]) -> String {
    if districts.is_empty() {
        return ALL_DISTRICTS.to_string();
    }
    let parts: Vec<String> = districts
        .iter()
        .map(|d| format!("{} ({})", d.region_number(), d.title(AnyLocale::Nl)))
        .collect();
    format!("kieskring {}", parts.join(", "))
}

/// An omission ("verzuim") signifies something was wrong with the submitted data
#[derive(Default, Debug, Serialize, Eq, PartialEq, Deserialize, Clone)]
pub struct Omission {
    pub id: OmissionId,
    pub category: OmissionCategory,
    /// Short title shown in the pill/badge layout
    pub title: String,
    /// The description for on the model I 1
    pub description: String,
    /// Help text for political groups explaining how to resolve the omission ("Dit verzuim is te herstellen door ...")
    pub help_text: String,
    #[serde(default = "recoverable_by_default")]
    pub recoverable: bool,
    pub updated_at: UtcDateTime,
}

impl Omission {
    pub fn new(
        category: OmissionCategory,
        title: String,
        description: String,
        help_text: String,
    ) -> Self {
        Omission {
            category,
            title,
            description,
            help_text,
            recoverable: true,
            ..Default::default()
        }
    }

    pub async fn create(&self, store: &CsbStore) -> Result<(), AppError> {
        store.update(CsbEvent::CreateOmission(self.clone())).await
    }

    pub async fn update(&self, store: &CsbStore) -> Result<(), AppError> {
        store.update(CsbEvent::UpdateOmission(self.clone())).await
    }

    pub async fn delete(&self, store: &CsbStore) -> Result<(), AppError> {
        store
            .update(CsbEvent::DeleteOmission {
                omission_id: self.id,
            })
            .await
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::CsbStore;

    pub fn sample_omission(category: OmissionCategory) -> Omission {
        Omission::new(
            category,
            "test title".to_string(),
            "test description".to_string(),
            "test help text".to_string(),
        )
    }

    #[test]
    fn presets_are_loaded_from_json_per_type() {
        assert_eq!(OmissionType::PoliticalGroup.presets(false).len(), 6);
        assert_eq!(OmissionType::CandidateList.presets(false).len(), 4);
        // A candidate omission draws from a different set depending on whether it
        // is scoped to the candidate on a specific list or general to the person.
        assert_eq!(OmissionType::Candidate.presets(false).len(), 3);
        assert_eq!(OmissionType::Candidate.presets(true).len(), 9);

        // Every preset carries a title and description; irreparable defects have
        // no help text.
        assert!(
            OmissionType::PoliticalGroup
                .presets(false)
                .iter()
                .all(|p| !p.title.is_empty() && !p.description.is_empty())
        );
        assert!(
            OmissionType::PoliticalGroup
                .presets(false)
                .iter()
                .any(|p| p.help_text.is_empty())
        );
    }

    #[test]
    fn presets_carry_the_recoverable_flag() {
        // Most omissions are recoverable ("herstelbaar").
        assert!(
            OmissionType::Candidate
                .presets(false)
                .iter()
                .any(|p| p.recoverable)
        );
        // Irreparable defects ("onherstelbaar verzuim") have no help text and are
        // flagged as non-recoverable.
        assert!(
            OmissionType::PoliticalGroup
                .presets(false)
                .iter()
                .any(|p| !p.recoverable)
        );
        assert!(
            OmissionType::PoliticalGroup
                .presets(false)
                .iter()
                .all(|p| p.recoverable || p.help_text.is_empty())
        );
    }

    #[test]
    fn omission_recoverable_defaults_to_true_for_legacy_events() {
        // Events persisted before the flag existed omit `recoverable`; they must
        // deserialize as recoverable rather than as errors.
        let json = r#"{
            "id": "00000000-0000-0000-0000-000000000000",
            "category": "PoliticalGroup",
            "title": "t",
            "description": "d",
            "help_text": "",
            "updated_at": "2026-01-01T00:00:00Z"
        }"#;
        let omission: Omission = serde_json::from_str(json).unwrap();
        assert!(omission.recoverable);
    }

    #[test]
    fn interpolate_fills_known_tokens_and_keeps_the_rest() {
        let placeholders = OmissionPlaceholders {
            candidate_number: Some("3".to_string()),
            candidate_name: Some("A.B. de Vries".to_string()),
            districts: None,
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

    #[tokio::test]
    async fn create_and_get_omission() -> Result<(), AppError> {
        let store = CsbStore::new_for_test();
        let omission = sample_omission(OmissionCategory::PoliticalGroup);

        omission.create(&store).await?;

        let loaded = store.get_omission(omission.id)?;
        assert_eq!(loaded.id, omission.id);
        assert_eq!(loaded.description, "test description");

        Ok(())
    }

    #[tokio::test]
    async fn update_omission_overwrites_fields() -> Result<(), AppError> {
        let store = CsbStore::new_for_test();
        let mut omission = sample_omission(OmissionCategory::PoliticalGroup);

        omission.create(&store).await?;

        omission.description = "Updated description".to_string();
        omission.update(&store).await?;

        let updated = store.get_omission(omission.id)?;
        assert_eq!(updated.description, "Updated description");

        Ok(())
    }

    #[tokio::test]
    async fn delete_omission_removes_record() -> Result<(), AppError> {
        let store = CsbStore::new_for_test();
        let omission = sample_omission(OmissionCategory::PoliticalGroup);

        omission.create(&store).await?;
        omission.delete(&store).await?;

        let missing = store.get_omission(omission.id);
        assert!(missing.is_err());

        Ok(())
    }

    mod electoral_district {
        use super::*;
        use crate::{ElectionConfig, ElectoralDistrict, candidate_lists::CandidateList};

        const EK: ElectionConfig = ElectionConfig::EK27;

        fn store_with_list(districts: Vec<ElectoralDistrict>) -> (CsbStore, CandidateListId) {
            let store = CsbStore::new_for_test();
            let list = CandidateList {
                electoral_districts: districts,
                ..Default::default()
            };
            let id = list.id;
            store.set_candidate_list(list);
            (store, id)
        }

        #[test]
        fn political_group_maps_to_all_districts() {
            let store = CsbStore::new_for_test();
            assert_eq!(
                OmissionCategory::PoliticalGroup
                    .electoral_district(&store, &EK)
                    .unwrap(),
                "alle kieskringen"
            );
        }

        #[test]
        fn candidate_without_list_maps_to_all_districts() {
            let store = CsbStore::new_for_test();
            let category = OmissionCategory::Candidate {
                person: crate::persons::PersonId::new(),
                list: None,
            };
            assert_eq!(
                category.electoral_district(&store, &EK).unwrap(),
                "alle kieskringen"
            );
        }

        #[test]
        fn candidate_list_with_all_districts_maps_to_all() {
            let store = CsbStore::new_for_test();
            assert_eq!(
                OmissionCategory::CandidateList(EK.electoral_districts().to_vec())
                    .electoral_district(&store, &EK)
                    .unwrap(),
                "alle kieskringen"
            );
        }

        #[test]
        fn candidate_list_with_one_district() {
            let store = CsbStore::new_for_test();
            assert_eq!(
                OmissionCategory::CandidateList(vec![ElectoralDistrict::BO])
                    .electoral_district(&store, &EK)
                    .unwrap(),
                "kieskring 13 (Bonaire)"
            );
        }

        #[test]
        fn candidate_list_with_multiple_districts() {
            let store = CsbStore::new_for_test();
            assert_eq!(
                OmissionCategory::CandidateList(vec![ElectoralDistrict::GR, ElectoralDistrict::DR])
                    .electoral_district(&store, &EK)
                    .unwrap(),
                "kieskring 1 (Groningen), 3 (Drenthe)"
            );
        }

        #[test]
        fn candidate_list_with_no_districts_maps_to_all() {
            let store = CsbStore::new_for_test();
            // An empty district list is treated as "all districts" in format_districts.
            assert_eq!(
                OmissionCategory::CandidateList(vec![])
                    .electoral_district(&store, &EK)
                    .unwrap(),
                "alle kieskringen"
            );
        }

        #[test]
        fn candidate_with_list_all_districts_maps_to_all() {
            let (store, id) = store_with_list(EK.electoral_districts().to_vec());
            let category = OmissionCategory::Candidate {
                person: crate::persons::PersonId::new(),
                list: Some(id),
            };
            assert_eq!(
                category.electoral_district(&store, &EK).unwrap(),
                "alle kieskringen"
            );
        }

        #[test]
        fn candidate_with_list_specific_district() {
            let (store, id) = store_with_list(vec![ElectoralDistrict::GR]);
            let category = OmissionCategory::Candidate {
                person: crate::persons::PersonId::new(),
                list: Some(id),
            };
            assert_eq!(
                category.electoral_district(&store, &EK).unwrap(),
                "kieskring 1 (Groningen)"
            );
        }

        #[test]
        fn candidate_with_missing_list_returns_error() {
            let store = CsbStore::new_for_test();
            let category = OmissionCategory::Candidate {
                person: crate::persons::PersonId::new(),
                list: Some(CandidateListId::new()),
            };
            assert!(category.electoral_district(&store, &EK).is_err());
        }
    }
}
