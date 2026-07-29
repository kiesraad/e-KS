use parking_lot::{
    RawRwLock,
    lock_api::{MappedRwLockReadGuard, RwLockReadGuard},
};

use crate::{
    AppError, CsbStore, PgStoreData,
    candidate_lists::{CandidateList, CandidateListId},
    list_submitters::ListSubmitter,
    name_authorisations::NameAuthorisation,
    persons::{Person, PersonId},
    political_groups::PoliticalGroup,
    structs::csb::{Omission, OmissionCategory, OmissionId},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WithCorrections {
    /// Only the original imported data
    None,
    /// Include corrections made in paper correction mode
    Paper,
    /// Also include CSB ("ambtshalve") corrections
    All,
}

impl CsbStore {
    pub fn read(
        &self,
        corrections: WithCorrections,
    ) -> MappedRwLockReadGuard<'_, RawRwLock, PgStoreData> {
        let data = self.data.read();

        match corrections {
            WithCorrections::None => RwLockReadGuard::map(data, |data| &data.imported_data),
            WithCorrections::Paper | WithCorrections::All => {
                // CSB corrections are applied on top of the paper corrected data by the appropriate getters
                RwLockReadGuard::map(data, |data| &data.paper_corrected_data)
            }
        }
    }

    pub fn is_examination_finished(&self) -> bool {
        let data = self.data.read();

        data.is_examination_finished
    }

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

    /// get the total number of CSB corrections added
    pub fn get_correction_count(&self) -> usize {
        let data = self.data.read();

        data.csb_corrected_persons
            .values()
            .map(|p| p.get_corrections().len())
            .sum::<usize>()
            + data.csb_corrected_display_name.as_ref().map_or(0, |_| 1)
    }

    /// get the total number of CSB corrections and omissions
    pub fn get_rectification_count(&self) -> usize {
        self.get_omission_count() + self.get_correction_count()
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

    /// Return all candidate-list omissions that reference the given list.
    pub fn get_candidate_list_omissions(
        &self,
        list_id: CandidateListId,
    ) -> Result<Vec<Omission>, AppError> {
        if self
            .get_candidate_list(list_id, WithCorrections::All)
            .is_none()
        {
            return Err(AppError::GenericNotFound);
        }

        let data = self.data.read();

        Ok(data
            .omissions
            .values()
            .filter(|o| {
                matches!(&o.category, OmissionCategory::CandidateList(lists)
                    if lists.contains(&list_id))
            })
            .cloned()
            .collect())
    }

    pub fn get_all_declarations_of_support_omissions(&self) -> Vec<Omission> {
        let data = self.data.read();

        data.omissions
            .values()
            .filter(|o| matches!(o.category, OmissionCategory::DeclarationsOfSupport(_)))
            .cloned()
            .collect()
    }

    pub fn get_political_group(&self, corrections: WithCorrections) -> PoliticalGroup {
        let mut pg = self.read(corrections).political_group.clone();

        if corrections == WithCorrections::All
            && let Some(correction) = self.data.read().csb_corrected_display_name.clone()
        {
            pg.display_name = Some(correction);
        }

        pg
    }

    pub fn get_candidate_lists(&self, corrections: WithCorrections) -> Vec<CandidateList> {
        self.read(corrections)
            .candidate_lists
            .values()
            .cloned()
            .collect()
    }

    /// The candidate list with this id, if any.
    pub fn get_candidate_list(
        &self,
        list_id: CandidateListId,
        corrections: WithCorrections,
    ) -> Option<CandidateList> {
        self.read(corrections)
            .candidate_lists
            .get(&list_id)
            .cloned()
    }

    /// The person (candidate) with this id, if any.
    pub fn get_person(&self, person_id: PersonId, corrections: WithCorrections) -> Option<Person> {
        let data = self.read(corrections);

        let person = data.persons.get(&person_id).cloned();

        if corrections == WithCorrections::All
            && let Some(mut person) = person.clone()
            && let Some(delta) = self
                .data
                .read()
                .csb_corrected_persons
                .get(&person_id)
                .cloned()
        {
            delta.apply(&mut person);
            Some(person)
        } else {
            person
        }
    }

    pub fn get_all_csb_corrected_persons(&self) -> Vec<PersonId> {
        self.data
            .read()
            .csb_corrected_persons
            .keys()
            .cloned()
            .collect()
    }

    /// The name of the first candidate across all candidate lists, sorted by list
    /// creation date. Returns `None` when no candidates are available.
    pub fn get_first_candidate_name(
        &self,
        corrections: WithCorrections,
    ) -> Option<crate::structs::common::FullName> {
        let mut lists = self.get_candidate_lists(corrections);
        lists.sort_unstable_by_key(|l| l.created_at);
        lists
            .into_iter()
            .flat_map(|list| list.candidates.into_iter())
            .next()
            .and_then(|id| self.get_person(id, corrections))
            .map(|p| p.name)
    }

    /// Short-hand to get the display name of the political group (including special names for blank lists)
    pub fn get_display_name(&self, corrections: WithCorrections) -> String {
        let political_group = self.get_political_group(corrections);
        political_group.csb_display_name(self.get_first_candidate_name(corrections).as_ref())
    }

    /// One-based position of the candidate on the given list
    pub fn get_candidate_position(
        &self,
        list_id: CandidateListId,
        person_id: PersonId,
        corrections: WithCorrections,
    ) -> Option<usize> {
        self.read(corrections)
            .candidate_lists
            .get(&list_id)?
            .position_of(person_id)
    }

    pub fn get_list_submitter(&self, corrections: WithCorrections) -> ListSubmitter {
        self.read(corrections).list_submitter.clone()
    }

    pub fn get_substitute_submitters(&self, corrections: WithCorrections) -> Vec<ListSubmitter> {
        ListSubmitter::clone_as_substitutes(&self.read(corrections).substitute_submitters)
    }

    pub fn get_name_authorisations(&self, corrections: WithCorrections) -> Vec<NameAuthorisation> {
        self.read(corrections)
            .name_authorisations
            .values()
            .cloned()
            .collect()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CsbStore, ElectoralDistrict,
        candidate_lists::CandidateList,
        list_designation::ListDesignation,
        structs::csb::{OmissionCategory, sample_omission},
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
            .insert(list_id, list.clone());
        store
            .data
            .write()
            .paper_corrected_data
            .candidate_lists
            .insert(list_id, list);
    }

    #[test]
    fn get_political_group_omissions_returns_only_political_group() {
        let store = CsbStore::new_for_test();
        insert(&store, OmissionCategory::PoliticalGroup);
        insert(
            &store,
            OmissionCategory::CandidateList(vec![CandidateListId::new()]),
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
            OmissionCategory::CandidateList(vec![CandidateListId::new()]),
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
    fn get_candidate_list_omissions_returns_omissions_referencing_that_list() {
        let list_a = CandidateListId::new();
        let list_b = CandidateListId::new();
        let store = CsbStore::new_for_test();
        insert_list(&store, list_a, vec![ElectoralDistrict::GR]);
        insert_list(&store, list_b, vec![ElectoralDistrict::DR]);
        insert(&store, OmissionCategory::CandidateList(vec![list_a]));
        insert(&store, OmissionCategory::CandidateList(vec![list_b]));
        insert(&store, OmissionCategory::PoliticalGroup);

        let result_a = store.get_candidate_list_omissions(list_a).unwrap();
        let result_b = store.get_candidate_list_omissions(list_b).unwrap();

        assert_eq!(result_a.len(), 1);
        assert!(
            matches!(&result_a[0].category, OmissionCategory::CandidateList(ids) if ids == &[list_a])
        );
        assert_eq!(result_b.len(), 1);
        assert!(
            matches!(&result_b[0].category, OmissionCategory::CandidateList(ids) if ids == &[list_b])
        );
    }

    #[test]
    fn get_candidate_list_prefers_the_paper_corrected_version() {
        let list_id = CandidateListId::new();
        let store = CsbStore::new_for_test();
        insert_list(&store, list_id, vec![ElectoralDistrict::UT]);
        store.set_paper_corrected_candidate_list(CandidateList {
            id: list_id,
            electoral_districts: vec![ElectoralDistrict::GR],
            ..Default::default()
        });

        let list = store
            .get_candidate_list(list_id, WithCorrections::None)
            .unwrap();
        assert_eq!(list.electoral_districts, vec![ElectoralDistrict::UT]);
        let list = store
            .get_candidate_list(list_id, WithCorrections::Paper)
            .unwrap();
        assert_eq!(list.electoral_districts, vec![ElectoralDistrict::GR]);
    }

    #[test]
    fn get_person_falls_back_to_a_paper_added_person() {
        let store = CsbStore::new_for_test();
        let person_id = PersonId::new();
        let person = sample_person_with(person_id, None, "Jansen", None, "A.B.");
        store
            .data
            .write()
            .paper_corrected_data
            .persons
            .insert(person_id, person);

        assert!(store.get_person(person_id, WithCorrections::None).is_none());
        assert!(
            store
                .get_person(person_id, WithCorrections::Paper)
                .is_some()
        );
    }

    #[test]
    fn get_candidate_list_omissions_errors_for_unknown_list() {
        let store = CsbStore::new_for_test();
        insert(
            &store,
            OmissionCategory::CandidateList(vec![CandidateListId::new()]),
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

        let result = store.get_substitute_submitters(WithCorrections::None);

        assert_eq!(result.len(), 1);
        assert!(result[0].is_substitute);
    }

    #[test]
    fn get_substitute_submitters_returns_empty_when_none() {
        let store = CsbStore::new_for_test();

        assert!(
            store
                .get_substitute_submitters(WithCorrections::All)
                .is_empty()
        );
    }

    #[test]
    fn csb_display_name_standalone_list_uses_display_name() {
        let store = CsbStore::new_for_test();
        store.set_political_group(PoliticalGroup {
            display_name: Some("Kiesraad Demo".parse().unwrap()),
            list_designation: Some(ListDesignation::Standalone),
            ..Default::default()
        });

        assert_eq!(
            store.get_display_name(WithCorrections::All),
            "Kiesraad Demo"
        );
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
        store.add_person(person);

        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.candidates.push(person_id);
        store.add_candidate_list(list);

        assert_eq!(
            store.get_display_name(WithCorrections::All),
            "Blanco (Jansen, A.B.)"
        );
    }

    #[test]
    fn csb_display_name_blank_list_without_candidates_uses_blanco_fallback() {
        let store = CsbStore::new_for_test();
        store.set_political_group(PoliticalGroup {
            list_designation: Some(ListDesignation::Blank),
            ..Default::default()
        });

        assert_eq!(store.get_display_name(WithCorrections::All), "Blanco");
    }
}
