use crate::{
    core::{ElectionType, ModelLocale},
    submit::structs::{TypstCandidate, TypstDatetime, TypstElectoralDistricts},
};
use serde::Serialize;

/// Data for the Typst templates shared by all the models
#[derive(Debug, Serialize)]
pub struct TypstModelData {
    pub election_name: String,
    pub election_type: ElectionType,
    pub electoral_districts: TypstElectoralDistricts,
    pub designation: String,
    pub candidates: Vec<TypstCandidate>,
    pub timestamp: TypstDatetime,
    pub locale: ModelLocale,
    pub event_id: usize,
    pub sha_hash: String,
}
