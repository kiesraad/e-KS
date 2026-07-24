mod preset;

pub use preset::OmissionPlaceholders;

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    ElectoralDistrict,
    form::ValidationError,
    id_newtype,
    structs::{candidate_lists::CandidateListId, common::UtcDateTime, persons::PersonId},
};

id_newtype!(pub struct OmissionId);

/// The kind of item an omission is added to, carried as a path parameter so a
/// single "add omission" dialog can serve political groups, candidate lists and
/// candidates. Maps to a concrete [`OmissionCategory`] together with a
/// referenced item id (see [`OmissionCategory::from_type_and_reference`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmissionType {
    PoliticalGroup,
    CandidateList,
    DeclarationsOfSupport,
    Candidate,
}

impl OmissionType {
    fn as_str(self) -> &'static str {
        match self {
            OmissionType::PoliticalGroup => "political-group",
            OmissionType::CandidateList => "candidate-list",
            OmissionType::DeclarationsOfSupport => "declarations-of-support",
            OmissionType::Candidate => "candidate",
        }
    }

    pub fn needs_districts(self) -> bool {
        matches!(self, OmissionType::DeclarationsOfSupport)
    }

    pub fn needs_candidate_lists(self) -> bool {
        matches!(self, OmissionType::CandidateList | OmissionType::Candidate)
    }
}

impl FromStr for OmissionType {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "political-group" => Ok(OmissionType::PoliticalGroup),
            "candidate-list" => Ok(OmissionType::CandidateList),
            "declarations-of-support" => Ok(OmissionType::DeclarationsOfSupport),
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
    /// Omissions scoped to one or more specific candidate lists.
    CandidateList(Vec<CandidateListId>),
    /// Missing or incorrect "ondersteuningsverklaringen" (H 4), per district.
    DeclarationsOfSupport(Vec<ElectoralDistrict>),
    /// E.g. missing or invalid candidate data, missing or invalid "instemmingsverklaring" (H 9),
    /// missing copy of identity document
    Candidate {
        person: PersonId,
        /// The candidate lists to which this applies.
        lists: Vec<CandidateListId>,
    },
}

impl OmissionCategory {
    /// Build the category for a newly added omission from the parameters of the
    /// "add omission" dialog. For `DeclarationsOfSupport`, construct the category
    /// directly with the selected districts (see `add_omission_submit`).
    pub fn from_type_and_reference(
        omission_type: OmissionType,
        reference: uuid::Uuid,
        lists: Vec<CandidateListId>,
    ) -> Self {
        match omission_type {
            OmissionType::PoliticalGroup => OmissionCategory::PoliticalGroup,
            OmissionType::CandidateList => OmissionCategory::CandidateList(lists),
            OmissionType::DeclarationsOfSupport => {
                unreachable!(
                    "DeclarationsOfSupport omissions must be created with explicit districts"
                )
            }
            OmissionType::Candidate => OmissionCategory::Candidate {
                person: reference.into(),
                lists,
            },
        }
    }
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

fn recoverable_by_default() -> bool {
    true
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

    pub fn class(&self) -> &str {
        if self.recoverable { "warning" } else { "error" }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{AppError, CsbStore};

    pub fn sample_omission(category: OmissionCategory) -> Omission {
        Omission::new(
            category,
            "test title".to_string(),
            "test description".to_string(),
            "test help text".to_string(),
        )
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
}
