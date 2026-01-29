use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{ElectoralDistrict, TokenValue, candidate_lists::CandidateList};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "CandidateList")]
#[serde(default)]
pub struct CandidateListForm {
    pub electoral_districts: Vec<ElectoralDistrict>,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<CandidateList> for CandidateListForm {
    fn from(value: CandidateList) -> Self {
        CandidateListForm {
            electoral_districts: value.electoral_districts,
            csrf_token: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CsrfTokens, ElectoralDistrict,
        form::{Validate, ValidationError},
    };

    #[test]
    fn builds_candidate_list_with_valid_csrf() {
        let tokens = CsrfTokens::default();
        let csrf_token = tokens.issue().value;
        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            csrf_token,
        };

        let list = form.validate_create(&tokens).unwrap();
        assert_eq!(list.electoral_districts, vec![ElectoralDistrict::UT]);
    }

    #[test]
    fn rejects_invalid_csrf_token() {
        let tokens = CsrfTokens::default();
        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            csrf_token: TokenValue("invalid".to_string()),
        };

        let Err(data) = form.validate_create(&tokens) else {
            panic!("expected validation errors");
        };

        assert!(
            data.errors()
                .contains(&("csrf_token".to_string(), ValidationError::InvalidCsrfToken))
        );
    }
}
