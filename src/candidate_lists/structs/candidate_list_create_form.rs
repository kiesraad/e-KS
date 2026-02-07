use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    CsrfToken, CsrfTokens, TokenValue,
    candidate_lists::CandidateList,
    form::{FormData, Validate, ValidationError, WithCsrfToken},
};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct CandidateListCreateForm {
    pub electoral_districts: Vec<crate::ElectoralDistrict>,
    pub copy_candidates: bool,
    pub csrf_token: TokenValue,
}

impl WithCsrfToken for CandidateListCreateForm {
    fn with_csrf_token(self, csrf_token: CsrfToken) -> Self {
        CandidateListCreateForm {
            csrf_token: csrf_token.value,
            ..self
        }
    }
}

impl Validate<CandidateList> for CandidateListCreateForm {
    fn validate_update(
        self,
        current: &CandidateList,
        csrf_tokens: &CsrfTokens,
    ) -> Result<CandidateList, FormData<Self>> {
        let mut errors = Vec::new();

        if !csrf_tokens.consume(&self.csrf_token) {
            errors.push(("csrf_token".to_string(), ValidationError::InvalidCsrfToken));
        }

        if !errors.is_empty() {
            return Err(FormData::new_with_errors(self, csrf_tokens, errors));
        }

        Ok(CandidateList {
            electoral_districts: self.electoral_districts,
            updated_at: Utc::now(),
            ..current.clone()
        })
    }

    fn validate_create(self, csrf_tokens: &CsrfTokens) -> Result<CandidateList, FormData<Self>> {
        let mut errors = Vec::new();

        if !csrf_tokens.consume(&self.csrf_token) {
            errors.push(("csrf_token".to_string(), ValidationError::InvalidCsrfToken));
        }

        if !errors.is_empty() {
            return Err(FormData::new_with_errors(self, csrf_tokens, errors));
        }

        Ok(CandidateList {
            electoral_districts: self.electoral_districts,
            updated_at: Utc::now(),
            created_at: Utc::now(),
            ..Default::default()
        })
    }
}
