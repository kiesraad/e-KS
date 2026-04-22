use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{ElectoralDistrict, TokenValue, candidate_lists::CandidateList};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "CandidateList")]
#[serde(default)]
pub struct CandidateListForm {
    #[validate(not_empty)]
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
        ElectoralDistrict,
        form::{ValidationError, generate_csrf_token},
    };

    #[tokio::test]
    async fn builds_candidate_list_with_valid_csrf() {
        let csrf_token = generate_csrf_token();
        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            csrf_token: csrf_token.clone(),
        };

        let list = form.validate_create(&csrf_token).unwrap();
        assert_eq!(list.electoral_districts, vec![ElectoralDistrict::UT]);
    }

    #[tokio::test]
    async fn rejects_invalid_csrf_token() {
        let csrf_token = generate_csrf_token();
        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            csrf_token: TokenValue("invalid".to_string()),
        };

        let Err(data) = form.validate_create(&csrf_token) else {
            panic!("expected validation errors");
        };

        assert!(
            data.errors()
                .contains(&("csrf_token".to_string(), ValidationError::InvalidCsrfToken))
        );
    }
}
