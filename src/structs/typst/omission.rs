use serde::Serialize;

/// Omissions for the I 4 sections 3, 4, and 5
#[derive(Debug, Default, Serialize)]
pub struct TypstOmission {
    pub designation: String,
    pub electoral_district: String,
    pub omission_descriptions: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct TypstRemovedCandidate {
    pub name: String,
    pub reason: String,
}

/// For the I 4 section "6. Geschrapte kandidaten"
#[derive(Debug, Default, Serialize)]
pub struct TypstRemovedCandidates {
    pub designation: String,
    pub electoral_district: String,
    pub candidates: Vec<TypstRemovedCandidate>,
}

/// For the I 4 section "7. Geschrapte aanduidingen"
#[derive(Debug, Default, Serialize)]
pub struct TypstRemovedDesignation {
    pub designation: String,
    pub electoral_district: String,
    pub first_candidate_name: String,
    pub reason: String,
}
