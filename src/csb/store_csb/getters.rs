use crate::{
    AppError, CsbStore,
    candidate_lists::{CandidateList, CandidateListId},
    csb::{Omission, OmissionId, omission::OmissionCategory},
    list_submitters::ListSubmitter,
    name_authorisations::NameAuthorisation,
    persons::{Person, PersonId},
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

    pub fn get_omission_count(&self) -> usize {
        let data = self.data.read();

        data.omissions.len()
    }

    pub fn get_recoverable_omissions(&self) -> Vec<Omission> {
        let data = self.data.read();

        data.omissions
            .values()
            .filter(|o| o.recoverable)
            .cloned()
            .collect()
    }

    pub fn get_political_group_omissions(&self) -> Vec<Omission> {
        let data = self.data.read();

        data.omissions
            .values()
            .filter(|o| matches!(o.category, OmissionCategory::PoliticalGroup))
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

    /// Return all candidate-list omissions that are linked to at least one
    /// electoral district covered by the given list.
    pub fn get_candidate_list_omissions(
        &self,
        list_id: CandidateListId,
    ) -> Result<Vec<Omission>, AppError> {
        let data = self.data.read();

        let list_districts = &data
            .imported_data
            .candidate_lists
            .get(&list_id)
            .ok_or(AppError::GenericNotFound)?
            .electoral_districts;

        Ok(data
            .omissions
            .values()
            .filter(|o| {
                matches!(&o.category, OmissionCategory::CandidateList(districts)
                    if districts.iter().any(|d| list_districts.contains(d)))
            })
            .cloned()
            .collect())
    }

    /// Return the single stored omission. Test-only helper for asserting on
    /// omissions whose category has no dedicated getter (e.g. candidate lists).
    #[cfg(test)]
    pub fn get_omission_for_test(&self) -> Omission {
        self.data
            .read()
            .omissions
            .values()
            .next()
            .cloned()
            .expect("expected exactly one stored omission")
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

    /// The imported candidate list with this id, if any.
    pub fn get_candidate_list(&self, list_id: CandidateListId) -> Option<CandidateList> {
        self.data
            .read()
            .imported_data
            .candidate_lists
            .get(&list_id)
            .cloned()
    }

    /// The imported person (candidate) with this id, if any.
    pub fn get_person(&self, person_id: PersonId) -> Option<Person> {
        self.data
            .read()
            .imported_data
            .persons
            .get(&person_id)
            .cloned()
    }

    /// The name of the first candidate across all imported candidate lists,
    /// sorted by list creation date. Returns `None` when no candidates are imported.
    pub fn first_candidate_name(&self) -> Option<crate::common::FullName> {
        let mut lists = self.get_candidate_lists();
        lists.sort_unstable_by_key(|l| l.created_at);
        lists
            .into_iter()
            .flat_map(|list| list.candidates.into_iter())
            .next()
            .and_then(|id| self.get_person(id))
            .map(|p| p.name)
    }

    /// Short-hand to get the display name of the political group
    pub fn csb_display_name(&self) -> String {
        let political_group = self.get_political_group();
        political_group.csb_display_name(self.first_candidate_name().as_ref())
    }

    pub fn candidate_position(
        &self,
        list_id: CandidateListId,
        person_id: PersonId,
    ) -> Option<usize> {
        let data = self.data.read();

        data.imported_data
            .candidate_lists
            .get(&list_id)?
            .candidates
            .iter()
            .position(|candidate| *candidate == person_id)
            .map(|index| index + 1)
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
        CsbStore, ElectoralDistrict,
        candidate_lists::CandidateList,
        csb::omission::{OmissionCategory, tests::sample_omission},
        list_designation::ListDesignation,
        test_utils::{sample_candidate_list, sample_person_with},
    };

    fn insert(store: &CsbStore, category: OmissionCategory) {
        let omission = sample_omission(category);
        store.data.write().omissions.insert(omission.id, omission);
    }

    fn insert_list(store: &CsbStore, list_id: CandidateListId, districts: Vec<ElectoralDistrict>) {
        let list = CandidateList {
            id: list_id,
            electoral_districts: districts,
            ..Default::default()
        };
        store
            .data
            .write()
            .imported_data
            .candidate_lists
            .insert(list_id, list);
    }

    #[test]
    fn get_political_group_omissions_returns_only_political_group() {
        let store = CsbStore::new_for_test();
        insert(&store, OmissionCategory::PoliticalGroup);
        insert(
            &store,
            OmissionCategory::CandidateList(vec![ElectoralDistrict::GR]),
        );

        let result = store.get_political_group_omissions();

        assert_eq!(result.len(), 1);
        assert!(matches!(
            result[0].category,
            OmissionCategory::PoliticalGroup
        ));
    }

    #[test]
    fn get_political_group_omissions_returns_empty_when_none() {
        let store = CsbStore::new_for_test();
        insert(
            &store,
            OmissionCategory::CandidateList(vec![ElectoralDistrict::GR]),
        );

        assert!(store.get_political_group_omissions().is_empty());
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
                lists: Vec::new(),
            },
        );
        insert(
            &store,
            OmissionCategory::Candidate {
                person: person_b,
                lists: Vec::new(),
            },
        );
        insert(&store, OmissionCategory::PoliticalGroup);

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
                lists: Vec::new(),
            },
        );

        assert!(store.get_candidate_omissions(PersonId::new()).is_empty());
    }

    #[test]
    fn get_candidate_list_omissions_returns_omissions_sharing_a_district_with_the_list() {
        let list_a = CandidateListId::new();
        let list_b = CandidateListId::new();
        let store = CsbStore::new_for_test();
        insert_list(&store, list_a, vec![ElectoralDistrict::GR]);
        insert_list(&store, list_b, vec![ElectoralDistrict::DR]);
        // Omission for GR: should appear in list_a but not list_b.
        insert(
            &store,
            OmissionCategory::CandidateList(vec![ElectoralDistrict::GR]),
        );
        // Omission for DR: should appear in list_b but not list_a.
        insert(
            &store,
            OmissionCategory::CandidateList(vec![ElectoralDistrict::DR]),
        );
        insert(&store, OmissionCategory::PoliticalGroup);

        let result_a = store.get_candidate_list_omissions(list_a).unwrap();
        let result_b = store.get_candidate_list_omissions(list_b).unwrap();

        assert_eq!(result_a.len(), 1);
        assert!(
            matches!(&result_a[0].category, OmissionCategory::CandidateList(d) if d == &[ElectoralDistrict::GR])
        );
        assert_eq!(result_b.len(), 1);
        assert!(
            matches!(&result_b[0].category, OmissionCategory::CandidateList(d) if d == &[ElectoralDistrict::DR])
        );
    }

    #[test]
    fn get_candidate_list_omissions_includes_omission_covering_multiple_districts() {
        let list_id = CandidateListId::new();
        let store = CsbStore::new_for_test();
        insert_list(&store, list_id, vec![ElectoralDistrict::GR]);
        // Omission for both GR and DR: overlaps with the list, so it should appear.
        insert(
            &store,
            OmissionCategory::CandidateList(vec![ElectoralDistrict::GR, ElectoralDistrict::DR]),
        );

        assert_eq!(
            store.get_candidate_list_omissions(list_id).unwrap().len(),
            1
        );
    }

    #[test]
    fn get_candidate_list_omissions_returns_empty_when_no_district_overlap() {
        let list_id = CandidateListId::new();
        let store = CsbStore::new_for_test();
        insert_list(&store, list_id, vec![ElectoralDistrict::GR]);
        insert(
            &store,
            OmissionCategory::CandidateList(vec![ElectoralDistrict::DR]),
        );

        assert!(
            store
                .get_candidate_list_omissions(list_id)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn get_candidate_list_omissions_errors_for_unknown_list() {
        let store = CsbStore::new_for_test();
        insert(
            &store,
            OmissionCategory::CandidateList(vec![ElectoralDistrict::GR]),
        );

        assert!(
            store
                .get_candidate_list_omissions(CandidateListId::new())
                .is_err()
        );
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

    #[test]
    fn csb_display_name_standalone_list_uses_display_name() {
        let store = CsbStore::new_for_test();
        store.set_political_group(PoliticalGroup {
            display_name: Some("Kiesraad Demo".parse().unwrap()),
            list_designation: Some(ListDesignation::Standalone),
            ..Default::default()
        });

        assert_eq!(store.csb_display_name(), "Kiesraad Demo");
    }

    #[test]
    fn csb_display_name_blank_list_with_candidate_uses_first_candidate_name() {
        let store = CsbStore::new_for_test();
        store.set_political_group(PoliticalGroup {
            list_designation: Some(ListDesignation::Blank),
            ..Default::default()
        });

        let person_id = PersonId::new();
        let person = sample_person_with(person_id, None, "Jansen", None, "A.B.");
        store.set_person(person);

        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates.push(person_id);
        store.set_candidate_list(list);

        assert_eq!(store.csb_display_name(), "Blanco (Jansen, A.B.)");
    }

    #[test]
    fn csb_display_name_blank_list_without_candidates_uses_blanco_fallback() {
        let store = CsbStore::new_for_test();
        store.set_political_group(PoliticalGroup {
            list_designation: Some(ListDesignation::Blank),
            ..Default::default()
        });

        assert_eq!(store.csb_display_name(), "Blanco");
    }
}
