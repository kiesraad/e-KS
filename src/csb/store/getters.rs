use crate::{
    AppError, CsbStore,
    csb::{Omission, OmissionId, omission::OmissionCategory},
    persons::PersonId,
};

impl CsbStore {
    pub fn get_omission(&self, omission_id: OmissionId) -> Result<Omission, AppError> {
        let data = self.data.read();

        data.omissions
            .get(&omission_id)
            .cloned()
            .ok_or(AppError::GenericNotFound)
    }

    pub fn get_general_omissions(&self) -> Vec<Omission> {
        let data = self.data.read();

        data.omissions
            .values()
            .filter(|o| matches!(o.category, OmissionCategory::General))
            .cloned()
            .collect()
    }

    pub fn get_name_authorisation_omissions(&self) -> Vec<Omission> {
        let data = self.data.read();

        data.omissions
            .values()
            .filter(|o| matches!(o.category, OmissionCategory::NameAuthorisation(_)))
            .cloned()
            .collect()
    }

    pub fn get_declaration_of_support_omissions(&self) -> Vec<Omission> {
        let data = self.data.read();

        data.omissions
            .values()
            .filter(|o| matches!(o.category, OmissionCategory::DeclarationOfSupport(_)))
            .cloned()
            .collect()
    }

    pub fn get_candidate_omissions(&self, person_id: PersonId) -> Vec<Omission> {
        let data = self.data.read();

        data.omissions
            .values()
            .filter(|o| matches!(&o.category, OmissionCategory::Candidate { person, .. } if *person == person_id))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CsbStore,
        csb::omission::{OmissionCategory, tests::sample_omission},
    };

    fn insert(store: &CsbStore, category: OmissionCategory) {
        let omission = sample_omission(category);
        store.data.write().omissions.insert(omission.id, omission);
    }

    #[test]
    fn get_general_omissions_returns_only_general() {
        let store = CsbStore::new_for_test();
        insert(&store, OmissionCategory::General);
        insert(&store, OmissionCategory::NameAuthorisation(None));

        let result = store.get_general_omissions();

        assert_eq!(result.len(), 1);
        assert!(matches!(result[0].category, OmissionCategory::General));
    }

    #[test]
    fn get_general_omissions_returns_empty_when_none() {
        let store = CsbStore::new_for_test();

        assert!(store.get_general_omissions().is_empty());
    }

    #[test]
    fn get_name_authorisation_omissions_returns_only_name_authorisation() {
        let store = CsbStore::new_for_test();
        insert(&store, OmissionCategory::NameAuthorisation(None));
        insert(&store, OmissionCategory::General);

        let result = store.get_name_authorisation_omissions();

        assert_eq!(result.len(), 1);
        assert!(matches!(
            result[0].category,
            OmissionCategory::NameAuthorisation(_)
        ));
    }

    #[test]
    fn get_declaration_of_support_omissions_returns_only_declaration_of_support() {
        let store = CsbStore::new_for_test();
        insert(&store, OmissionCategory::DeclarationOfSupport(vec![]));
        insert(&store, OmissionCategory::General);

        let result = store.get_declaration_of_support_omissions();

        assert_eq!(result.len(), 1);
        assert!(matches!(
            result[0].category,
            OmissionCategory::DeclarationOfSupport(_)
        ));
    }

    #[test]
    fn get_candidate_omissions_returns_only_omissions_for_the_given_person() {
        let person_a = PersonId::new();
        let person_b = PersonId::new();
        let store = CsbStore::new_for_test();
        insert(
            &store,
            OmissionCategory::Candidate {
                person: person_a,
                list: None,
            },
        );
        insert(
            &store,
            OmissionCategory::Candidate {
                person: person_b,
                list: None,
            },
        );
        insert(&store, OmissionCategory::General);

        let result = store.get_candidate_omissions(person_a);

        assert_eq!(result.len(), 1);
        assert!(
            matches!(result[0].category, OmissionCategory::Candidate { person, .. } if person == person_a)
        );
    }

    #[test]
    fn get_candidate_omissions_returns_empty_when_no_match() {
        let store = CsbStore::new_for_test();
        insert(
            &store,
            OmissionCategory::Candidate {
                person: PersonId::new(),
                list: None,
            },
        );

        assert!(store.get_candidate_omissions(PersonId::new()).is_empty());
    }
}
