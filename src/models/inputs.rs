//! Input data types shared by the PDF models.
//!
//! Conversions from application/store types live in
//! `src/app/finalise/structs/`; type-checked example values live in
//! `super::examples`.

use crate::core::{ElectionType, ModelLocale};

/// Input data shared by the H models.
#[derive(Debug, Clone)]
pub struct ModelData {
    pub election_name: String,
    pub election_type: ModelElectionType,
    pub designation: String,
    pub candidates: Vec<Candidate>,
    pub locale: ModelLocale,
    pub event_id: usize,
    pub sha_hash: String,
}

/// The election type as used by the models. Unlike [`ElectionType`] this
/// distinguishes the electoral college for non-residents (`KCNI`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelElectionType {
    Tk,
    Ek,
    Gr,
    Ps,
    Ws,
    Ep,
    Kc,
    Kcni,
    Er,
}

impl From<ElectionType> for ModelElectionType {
    fn from(election_type: ElectionType) -> Self {
        match election_type {
            ElectionType::Tk => Self::Tk,
            ElectionType::Ek => Self::Ek,
            ElectionType::Gr => Self::Gr,
            ElectionType::Ps => Self::Ps,
            ElectionType::Ws => Self::Ws,
            ElectionType::Ep => Self::Ep,
            ElectionType::Kc => Self::Kc,
            ElectionType::Er => Self::Er,
        }
    }
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
    pub date_of_birth: Date,
    pub locality: String,
    pub position: usize,
}

/// A date, as year/month/day components.
#[derive(Debug, Clone)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_election_type_maps_to_a_model_election_type() {
        use ElectionType::*;
        // KCNI has no `ElectionType`; every other model type is reachable.
        let mapped: Vec<ModelElectionType> = [Tk, Ek, Gr, Ps, Ws, Ep, Kc, Er]
            .into_iter()
            .map(ModelElectionType::from)
            .collect();
        assert_eq!(mapped.first(), Some(&ModelElectionType::Tk));
        assert_eq!(mapped.last(), Some(&ModelElectionType::Er));
        assert!(!mapped.contains(&ModelElectionType::Kcni));
    }
}
