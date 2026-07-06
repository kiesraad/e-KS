use crate::{
    AppError, CsbStore,
    candidate_lists::CandidateList,
    csb::{Omission, OmissionId, omission::OmissionCategory},
    list_submitters::ListSubmitter,
    name_authorisations::NameAuthorisation,
    persons::PersonId,
    political_groups::PoliticalGroup,
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

    pub fn is_examination_finished(&self) -> bool {
        let data = self.data.read();

        data.is_examination_finished
    }

    pub fn get_political_group(&self) -> PoliticalGroup {
        let data = self.data.read();

        data.imported_data.political_group.clone()
    }

    pub fn get_candidate_lists(&self) -> Vec<CandidateList> {
        let data = self.data.read();

        data.imported_data
            .candidate_lists
            .values()
            .cloned()
            .collect()
    }

    /// The list submitter ("lijstinleveraar") imported for this political group.
    pub fn get_list_submitter(&self) -> ListSubmitter {
        let data = self.data.read();

        data.imported_data.list_submitter.clone()
    }

    /// The substitutes for the restoration of omissions ("vervangers voor het
    /// herstel van verzuimen") that were imported for this political group.
    pub fn get_substitute_submitters(&self) -> Vec<ListSubmitter> {
        let data = self.data.read();

        data.imported_data
            .substitute_submitters
            .iter()
            .cloned()
            .map(|mut submitter| {
                submitter.is_substitute = true;
                submitter
            })
            .collect()
    }

    /// The authorised names ("statutaire namen") imported for this political
    /// group.
    pub fn get_name_authorisations(&self) -> Vec<NameAuthorisation> {
        let data = self.data.read();

        data.imported_data
            .name_authorisations
            .values()
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

    #[test]
    fn get_substitute_submitters_marks_each_as_substitute() {
        let store = CsbStore::new_for_test();
        store
            .data
            .write()
            .imported_data
            .substitute_submitters
            .push(ListSubmitter::default());

        let result = store.get_substitute_submitters();

        assert_eq!(result.len(), 1);
        assert!(result[0].is_substitute);
    }

    #[test]
    fn get_substitute_submitters_returns_empty_when_none() {
        let store = CsbStore::new_for_test();

        assert!(store.get_substitute_submitters().is_empty());
    }
}
