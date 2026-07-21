mod preset;

pub use preset::OmissionPlaceholders;

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    AnyLocale, AppError, CsbEvent, CsbStore, ElectionConfig, ElectoralDistrict,
    candidate_lists::CandidateListId, common::UtcDateTime, form::ValidationError, id_newtype,
    persons::PersonId,
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
        /// The candidate lists to which this applies.
        lists: Vec<CandidateListId>,
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
        lists: Vec<CandidateListId>,
    ) -> Self {
        match omission_type {
            OmissionType::PoliticalGroup => OmissionCategory::PoliticalGroup,
            OmissionType::CandidateList => {
                unreachable!("CandidateList omissions must be created with explicit districts")
            }
            OmissionType::Candidate => OmissionCategory::Candidate {
                person: reference.into(),
                lists,
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
            OmissionCategory::PoliticalGroup => Ok(ALL_DISTRICTS.to_string()),
            OmissionCategory::CandidateList(districts) => Ok(format_districts(districts, election)),
            OmissionCategory::Candidate { lists, .. } => {
                let mut districts: Vec<ElectoralDistrict> = Vec::new();
                for id in lists {
                    let list = store
                        .get_candidate_list(*id)
                        .ok_or(AppError::GenericNotFound)?;
                    for d in list.electoral_districts {
                        if !districts.contains(&d) {
                            districts.push(d);
                        }
                    }
                }
                Ok(format_districts(&districts, election))
            }
        }
    }
}

fn format_districts(districts: &[ElectoralDistrict], election: &ElectionConfig) -> String {
    let all_districts = election.electoral_districts();
    if districts.is_empty() || all_districts.iter().all(|d| districts.contains(d)) {
        ALL_DISTRICTS.to_string()
    } else {
        let mut sorted = districts.to_vec();
        sorted.sort_by_key(|d| d.region_number());
        let parts: Vec<String> = sorted
            .iter()
            .map(|d| format!("{} ({})", d.region_number(), d.title(AnyLocale::Nl)))
            .collect();
        format!("kieskring {}", parts.join(", "))
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
    pub fn class(&self) -> &str {
        if self.recoverable { "warning" } else { "error" }
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
            store.add_candidate_list(list);
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
        fn candidate_with_all_districts_maps_to_all() {
            let (store, id) = store_with_list(EK.electoral_districts().to_vec());
            let category = OmissionCategory::Candidate {
                person: crate::persons::PersonId::new(),
                lists: vec![id],
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
            // The districts should be sorted by region number.
            assert_eq!(
                OmissionCategory::CandidateList(vec![ElectoralDistrict::DR, ElectoralDistrict::GR])
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
        fn candidate_with_list_specific_district() {
            let (store, id) = store_with_list(vec![ElectoralDistrict::GR]);
            let category = OmissionCategory::Candidate {
                person: crate::persons::PersonId::new(),
                lists: vec![id],
            };
            assert_eq!(
                category.electoral_district(&store, &EK).unwrap(),
                "kieskring 1 (Groningen)"
            );
        }

        #[test]
        fn candidate_with_paper_added_list_uses_the_corrected_projection() {
            let store = CsbStore::new_for_test();
            let list = CandidateList {
                electoral_districts: vec![ElectoralDistrict::GR],
                ..Default::default()
            };
            let id = list.id;
            store.set_paper_corrected_candidate_list(list);
            let category = OmissionCategory::Candidate {
                person: crate::persons::PersonId::new(),
                lists: vec![id],
            };

            assert_eq!(
                category.electoral_district(&store, &EK).unwrap(),
                "kieskring 1 (Groningen)"
            );
        }

        #[test]
        fn candidate_with_corrected_list_uses_the_corrected_districts() {
            let (store, id) = store_with_list(vec![ElectoralDistrict::UT]);
            store.set_paper_corrected_candidate_list(CandidateList {
                id,
                electoral_districts: vec![ElectoralDistrict::GR],
                ..Default::default()
            });
            let category = OmissionCategory::Candidate {
                person: crate::persons::PersonId::new(),
                lists: vec![id],
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
                lists: vec![CandidateListId::new()],
            };
            assert!(category.electoral_district(&store, &EK).is_err());
        }
    }
}
