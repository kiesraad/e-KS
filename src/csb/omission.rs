use serde::{Deserialize, Serialize};

use crate::{
    AppError, CsbEvent, CsbStore, ElectoralDistrict, candidate_lists::CandidateListId,
    common::UtcDateTime, id_newtype, name_authorisations::NameAuthorisationId, persons::PersonId,
};

id_newtype!(pub struct OmissionId);

#[derive(Default, Debug, Serialize, Eq, PartialEq, Deserialize, Clone)]
pub enum OmissionCategory {
    /// E.g. missing deposit ("waarborgsom"), unidentified submitter
    #[default]
    General,
    /// Missing, invalid or unregistered authorised agent and/or statutory name (H 3-1 / H 3-2)
    NameAuthorisation(Option<NameAuthorisationId>),
    /// Missing or incorrect "ondersteuningsverklaringen" for some "kieskringen" (H 4)
    DeclarationOfSupport(Vec<ElectoralDistrict>),
    /// E.g. too many candidates on a list
    CandidateList(CandidateListId),
    /// E.g. missing or invalid candidate data, missing or invalid "instemmingsverklaring" (H 9),
    /// missing copy of identity document
    Candidate {
        person: PersonId,
        /// The candidate list to which this applies, leave None if it applies to all candidate lists
        list: Option<CandidateListId>,
    },
}

/// An omission ("verzuim") signifies something was wrong with the submitted data
#[derive(Default, Debug, Serialize, Eq, PartialEq, Deserialize, Clone)]
pub struct Omission {
    pub id: OmissionId,
    pub category: OmissionCategory,
    /// The description for on the model I 1
    pub description: String,
    /// Help text for political groups explaining how to resolve the omission ("Dit verzuim is te herstellen door ...")
    pub help_text: String,
    pub updated_at: UtcDateTime,
}

impl Omission {
    pub fn new(category: OmissionCategory, description: String, help_text: String) -> Self {
        Omission {
            category,
            description,
            help_text,
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
            "test description".to_string(),
            "test help text".to_string(),
        )
    }

    #[tokio::test]
    async fn create_and_get_omission() -> Result<(), AppError> {
        let store = CsbStore::new_for_test();
        let omission = sample_omission(OmissionCategory::General);

        omission.create(&store).await?;

        let loaded = store.get_omission(omission.id)?;
        assert_eq!(loaded.id, omission.id);
        assert_eq!(loaded.description, "test description");

        Ok(())
    }

    #[tokio::test]
    async fn update_omission_overwrites_fields() -> Result<(), AppError> {
        let store = CsbStore::new_for_test();
        let mut omission = sample_omission(OmissionCategory::General);

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
        let omission = sample_omission(OmissionCategory::General);

        omission.create(&store).await?;
        omission.delete(&store).await?;

        let missing = store.get_omission(omission.id);
        assert!(missing.is_err());

        Ok(())
    }
}
