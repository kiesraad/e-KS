use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ElectoralDistrict, candidate_lists::structs::CandidateList, form::WithCsrfToken};
use validate::Validate as ValidateDerive;

#[derive(Default, Serialize, Deserialize, Clone, Debug, ValidateDerive)]
#[validate(
    target = "CandidateList",
    build = "CandidateListForm::build_candidate_list"
)]
#[serde(default)]
pub struct CandidateListForm {
    pub electoral_districts: Vec<ElectoralDistrict>,
    #[validate(csrf)]
    pub csrf_token: String,
}

impl WithCsrfToken for CandidateListForm {
    fn with_csrf_token(self, csrf_token: crate::form::CsrfToken) -> Self {
        CandidateListForm {
            csrf_token: csrf_token.value,
            ..self
        }
    }
}

impl CandidateListForm {
    fn build_candidate_list(
        validated: CandidateListFormValidated,
        current: Option<&CandidateList>,
    ) -> CandidateList {
        if let Some(current) = current {
            current.clone()
        } else {
            CandidateList {
                id: Uuid::new_v4(),
                electoral_districts: validated.electoral_districts,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }
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

        let list = form.validate(None, &tokens).unwrap();
        assert_eq!(list.electoral_districts, vec![ElectoralDistrict::UT]);
    }

    #[test]
    fn rejects_invalid_csrf_token() {
        let tokens = CsrfTokens::default();
        let form = CandidateListForm {
            electoral_districts: vec![ElectoralDistrict::UT],
            csrf_token: "invalid".to_string(),
        };

        let Err(data) = form.validate(None, &tokens) else {
            panic!("expected validation errors");
        };

        assert!(
            data.errors()
                .contains(&("csrf_token".to_string(), ValidationError::InvalidCsrfToken))
        );
    }
}
