use crate::{
    core::{ElectionType, ModelLocale},
    typst::{TypstCandidate, TypstDatetime, TypstElectoralDistricts},
};
use serde::Serialize;

/// Data for the Typst templates shared by all the PG models (H1/3/4/9)
#[derive(Debug, Serialize)]
pub struct TypstPgModelData {
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
