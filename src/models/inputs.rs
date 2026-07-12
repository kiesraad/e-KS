//! Input data types shared by the PDF models.
//!
//! Field names follow the JSON example inputs in `models/example-inputs`;
//! conversions from application/store types live in
//! `src/app/finalise/structs/`.

use serde::Deserialize;

use crate::core::{ElectionType, ModelLocale};

/// Input data shared by the H models.
#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "tag", content = "districts")]
pub enum ElectoralDistricts {
    All,
    Some(Vec<String>),
    /// The election has only one district, so the models omit the section.
    OnlyOne,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Candidate {
    pub last_name: String,
    /// Initials as printed on the model, e.g., optionally including the gender
    /// and first name
    pub initials: String,
    pub date_of_birth: Date,
    pub locality: String,
    pub position: usize,
}

/// A date, deserializable from `{year, month, day}` or an ISO `yyyy-mm-dd`
/// string (both occur in the example inputs).
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "DateRepr")]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DateRepr {
    Parts { year: i32, month: u32, day: u32 },
    Iso(String),
}

impl TryFrom<DateRepr> for Date {
    type Error = String;

    fn try_from(repr: DateRepr) -> Result<Self, Self::Error> {
        match repr {
            DateRepr::Parts { year, month, day } => Ok(Date { year, month, day }),
            DateRepr::Iso(iso) => {
                let mut parts = iso.split('-');
                let mut next = |name: &str| -> Result<u32, String> {
                    parts
                        .next()
                        .and_then(|part| part.parse().ok())
                        .ok_or_else(|| format!("invalid date `{iso}`: bad {name}"))
                };
                Ok(Date {
                    year: next("year")? as i32,
                    month: next("month")?,
                    day: next("day")?,
                })
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Person {
    pub last_name: String,
    /// Initials as printed on the model, e.g., optionally including the first name
    pub initials: String,
    /// Optional in the inputs: H 3 only prints the submitter's name.
    #[serde(default)]
    pub postal_address: PostalAddress,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PostalAddress {
    pub street_address: String,
    pub postal_code: String,
    pub locality: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NameAuthorisation {
    pub last_name: String,
    /// Initials as printed on the model, e.g., optionally including the first name
    pub initials: String,
    pub legal_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DetailedCandidate {
    pub candidate: Candidate,
    pub initials_no_gender: String,
    pub bsn: Option<String>,
    pub representative: Option<Person>,
    pub postal_address: Option<PostalAddress>,
}
