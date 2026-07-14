//! Input data types shared by the PDF models.
//!
//! Conversions from application/store types live in
//! `src/app/finalise/structs/`; type-checked example values live in
//! `super::examples`.

use chrono::NaiveDate;

use crate::core::{ElectionType, ModelLocale};

/// Input data shared by the H models.
#[derive(Debug, Clone)]
pub struct ModelData {
    pub election_name: String,
    pub election_type: ElectionType,
    pub designation: String,
    pub candidates: Vec<Candidate>,
    pub locale: ModelLocale,
    pub event_id: usize,
    pub sha_hash: String,
}

/// The electoral districts a candidate list applies to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElectoralDistricts {
    All,
    Some(Vec<String>),
    /// The election has only one district, so the models omit the section.
    OnlyOne,
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub last_name: String,
    /// Initials as printed on the model, e.g., optionally including the gender
    /// and first name
    pub initials: String,
    pub date_of_birth: NaiveDate,
    pub locality: String,
    pub position: usize,
}

#[derive(Debug, Clone)]
pub struct Person {
    pub last_name: String,
    /// Initials as printed on the model, e.g., optionally including the first name
    pub initials: String,
    /// Optional in the inputs: H 3 only prints the submitter's name, so this is
    /// left at its default there.
    pub postal_address: PostalAddress,
}

#[derive(Debug, Clone, Default)]
pub struct PostalAddress {
    pub street_address: String,
    pub postal_code: String,
    pub locality: String,
}

#[derive(Debug, Clone, Default)]
pub struct NameAuthorisation {
    pub last_name: String,
    /// Initials as printed on the model, e.g., optionally including the first name
    pub initials: String,
    pub legal_name: String,
}

#[derive(Debug, Clone)]
pub struct DetailedCandidate {
    pub candidate: Candidate,
    pub initials_no_gender: String,
    pub bsn: Option<String>,
    pub representative: Option<Person>,
    pub postal_address: Option<PostalAddress>,
}
