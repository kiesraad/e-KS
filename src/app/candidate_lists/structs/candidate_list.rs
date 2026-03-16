use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AppError, AppEvent, AppStore, ElectionConfig, ElectoralDistrict,
    candidate_lists::FullCandidateList,
    candidates::Candidate,
    common::UtcDateTime,
    core::AnyLocale,
    id_newtype,
    list_submitters::ListSubmitterId,
    persons::{Person, PersonId},
};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct CandidateListId);

#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CandidateList {
    pub id: CandidateListId,
    pub electoral_districts: Vec<ElectoralDistrict>,
    pub candidates: Vec<PersonId>,
    pub list_submitter_id: Option<ListSubmitterId>,
    pub substitute_list_submitter_ids: Vec<ListSubmitterId>,
    pub created_at: UtcDateTime,
}

impl CandidateList {
    pub fn districts_name(&self, locale: AnyLocale) -> String {
        self.electoral_districts
            .iter()
            .map(|d| d.title(locale))
            .collect::<Vec<&str>>()
            .join(", ")
    }

    pub fn districts_codes(&self) -> String {
        self.electoral_districts
            .iter()
            .map(|d| d.code().to_lowercase())
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn contains_all_districts(&self, election: &ElectionConfig) -> bool {
        self.electoral_districts.len() == election.electoral_districts().len()
    }

    pub fn used_districts(store: &AppStore) -> Result<Vec<ElectoralDistrict>, AppError> {
        let used: BTreeSet<ElectoralDistrict> = store
            .get_candidate_lists()
            .into_iter()
            .flat_map(|list| list.electoral_districts.into_iter())
            .collect();

        Ok(used.into_iter().collect())
    }

    pub fn available_districts(
        store: &AppStore,
        election: &ElectionConfig,
    ) -> Vec<ElectoralDistrict> {
        let used = CandidateList::used_districts(store).unwrap_or_default();

        election.available_districts(used)
    }

    pub async fn update_order(
        &mut self,
        store: &AppStore,
        person_ids: &[PersonId],
    ) -> Result<(), AppError> {
        let existing_person_ids = store
            .get_persons()
            .iter()
            .map(|p| p.id)
            .collect::<BTreeSet<_>>();

        // check all new ids exist
        if !person_ids.iter().all(|id| existing_person_ids.contains(id)) {
            return Err(AppError::GenericNotFound);
        }

        store.get_candidate_list(self.id)?;

        store
            .update(AppEvent::UpdateCandidateListOrder {
                list_id: self.id,
                candidates: person_ids.to_vec(),
            })
            .await?;

        *self = store.get_candidate_list(self.id)?;

        Ok(())
    }

    pub async fn update_position(
        &mut self,
        store: &AppStore,
        id: PersonId,
        position: usize,
    ) -> Result<(), AppError> {
        let Some(current_index) = self.candidates.iter().position(|&pid| pid == id) else {
            return Ok(());
        };

        let moved = self.candidates.remove(current_index);

        // convert the position (1, 2, 3...) to an index (0, 1, 2,..) and clamp it to the valid range
        let target_index = position.saturating_sub(1).min(self.candidates.len());

        self.candidates.insert(target_index, moved);

        self.update_order(store, &self.candidates.clone()).await?;

        Ok(())
    }

    pub async fn append_candidate(
        &mut self,
        store: &AppStore,
        person_id: PersonId,
    ) -> Result<(), AppError> {
        let person = store.get_person(person_id)?;

        if !self.candidates.contains(&person.id) {
            store
                .update(AppEvent::AddCandidateToCandidateList {
                    list_id: self.id,
                    person_id: person.id,
                })
                .await?;

            *self = store.get_candidate_list(self.id)?;
        }

        Ok(())
    }

    pub async fn remove_candidate(
        &mut self,
        store: &AppStore,
        person_id: PersonId,
    ) -> Result<(), AppError> {
        if self.candidates.contains(&person_id) {
            store
                .update(AppEvent::RemoveCandidateFromCandidateList {
                    list_id: self.id,
                    person_id,
                })
                .await?;

            *self = store.get_candidate_list(self.id)?;
        }

        Ok(())
    }

    pub async fn get_candidate(
        &self,
        store: &AppStore,
        person_id: PersonId,
    ) -> Result<Candidate, AppError> {
        let list = store.get_candidate_list(self.id)?;

        let position = list
            .candidates
            .iter()
            .position(|id| *id == person_id)
            .map(|index| index + 1)
            .ok_or_else(|| AppError::GenericNotFound)?;

        let person = store.get_person(person_id)?;

        Ok(Candidate {
            list_id: self.id,
            position,
            person,
        })
    }

    pub fn persons_not_on_list(
        &self,
        store: &AppStore,
        include: &[PersonId],
    ) -> Result<Vec<Person>, AppError> {
        let list = store.get_candidate_list(self.id)?;
        let existing: BTreeMap<PersonId, ()> =
            list.candidates.into_iter().map(|id| (id, ())).collect();

        Ok(store
            .get_sorted_persons()
            .into_iter()
            .filter(|person| !existing.contains_key(&person.id) || include.contains(&person.id))
            .collect())
    }

    pub async fn create(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::CreateCandidateList(self.clone()))
            .await
    }

    pub async fn update_districts(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateCandidateListDistricts {
                list_id: self.id,
                electoral_districts: self.electoral_districts.clone(),
            })
            .await
    }

    pub async fn update_submitters(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateCandidateListSubmitters {
                list_id: self.id,
                list_submitter_id: self.list_submitter_id,
                substitute_list_submitter_ids: self.substitute_list_submitter_ids.clone(),
            })
            .await
    }

    pub async fn delete(&self, store: &AppStore) -> Result<(), AppError> {
        store.update(AppEvent::DeleteCandidateList(self.id)).await
    }

    pub fn select_default_submitters(&mut self, store: &AppStore) -> Result<(), AppError> {
        if self.list_submitter_id.is_none() {
            self.list_submitter_id = store
                .get_list_submitters()
                .first()
                .map(|submitter| submitter.id);
        }

        if self.substitute_list_submitter_ids.is_empty() {
            self.substitute_list_submitter_ids = store
                .get_substitute_submitters()
                .iter()
                .map(|submitter| submitter.id)
                .collect();
        }

        Ok(())
    }

    pub(crate) fn build_full_candidate_list(
        store: &AppStore,
        list: CandidateList,
    ) -> Result<FullCandidateList, AppError> {
        let candidates = list
            .candidates
            .iter()
            .enumerate()
            .map(|(index, person_id)| {
                let person = store.get_person(*person_id)?;
                Ok(Candidate {
                    list_id: list.id,
                    position: index + 1,
                    person,
                })
            })
            .collect::<Result<Vec<Candidate>, AppError>>()?;

        Ok(FullCandidateList { list, candidates })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppStore,
        candidate_lists::CandidateListSummary,
        persons::PersonId,
        test_utils::{sample_candidate_list, sample_list_submitter, sample_person_with_last_name},
    };
    fn base_candidate_list(electoral_districts: Vec<ElectoralDistrict>) -> CandidateList {
        CandidateList {
            electoral_districts,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn districts_formats_titles_in_order() {
        let list = base_candidate_list(vec![
            ElectoralDistrict::UT,
            ElectoralDistrict::NH,
            ElectoralDistrict::DR,
        ]);

        assert_eq!(
            list.districts_name(AnyLocale::Nl),
            "Utrecht, Noord-Holland, Drenthe"
        );
        assert_eq!(
            list.districts_name(AnyLocale::En),
            "Utrecht, North Holland, Drenthe"
        );
    }

    #[tokio::test]
    async fn contains_all_districts_compares_to_election_config_length() {
        let election = ElectionConfig::EK2027;
        let list = base_candidate_list(election.electoral_districts().to_vec());
        assert!(list.contains_all_districts(&election));

        let list = base_candidate_list(vec![ElectoralDistrict::UT, ElectoralDistrict::NH]);
        assert!(!list.contains_all_districts(&election));
    }

    async fn insert_list(
        store: &AppStore,
        electoral_districts: Vec<ElectoralDistrict>,
    ) -> Result<CandidateList, AppError> {
        let list = CandidateList {
            electoral_districts,
            ..Default::default()
        };

        list.create(store).await?;

        Ok(list)
    }

    #[tokio::test]
    async fn create_and_list_candidate_lists() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());

        list.create(&store).await?;

        let lists = CandidateListSummary::list(&store)?;
        assert_eq!(1, lists.len());
        assert_eq!(list.id, lists[0].list.id);
        assert_eq!(0, lists[0].person_count);
        assert_eq!(0, lists[0].duplicate_districts.len());

        Ok(())
    }

    #[tokio::test]
    async fn get_candidate_list_summaries_with_duplicate_districts() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        // setup
        let list1 = insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        let list2 = insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::GR]).await?;

        let list3 = insert_list(&store, vec![ElectoralDistrict::OV, ElectoralDistrict::GR]).await?;

        // test
        let lists = CandidateListSummary::list(&store)?;

        // verification
        assert_eq!(3, lists.len());

        let list_summary1 = lists.iter().find(|list| list.list.id == list1.id).unwrap();
        let list_summary2 = lists.iter().find(|list| list.list.id == list2.id).unwrap();
        let list_summary3 = lists.iter().find(|list| list.list.id == list3.id).unwrap();

        // list 1 clashes on UT with list 2
        assert_eq!(
            vec![ElectoralDistrict::UT],
            list_summary1.duplicate_districts
        );

        // list 2 clashes on UT with list 1 and on GR with list 3
        assert_eq!(2, list_summary2.duplicate_districts.len());
        assert!(
            list_summary2
                .duplicate_districts
                .contains(&ElectoralDistrict::UT)
        );
        assert!(
            list_summary2
                .duplicate_districts
                .contains(&ElectoralDistrict::GR)
        );

        // list 3 clashes on GR with list 2
        assert_eq!(
            vec![ElectoralDistrict::GR],
            list_summary3.duplicate_districts
        );

        Ok(())
    }

    #[tokio::test]
    async fn list_candidate_list_orders_by_created_at() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_early = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            ..Default::default()
        };
        list_early.create(&store).await?;

        // sleep for a second to ensure a different created_at timestamp for the next list
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let list_late = CandidateList {
            electoral_districts: vec![ElectoralDistrict::OV],
            ..Default::default()
        };
        list_late.create(&store).await?;

        let lists = store.get_candidate_lists();
        assert_eq!(lists.len(), 2);
        assert_eq!(lists[0].id, list_early.id);
        assert_eq!(lists[1].id, list_late.id);

        Ok(())
    }

    #[tokio::test]
    async fn get_candidate_list_returns_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());

        list.create(&store).await?;

        let loaded = store.get_candidate_list(list.id)?;

        assert_eq!(loaded.id, list.id);

        Ok(())
    }

    #[tokio::test]
    async fn select_default_submitters_uses_store_defaults() -> Result<(), AppError> {
        let store = AppStore::new_for_test();

        let list_submitter_a =
            sample_list_submitter(crate::list_submitters::ListSubmitterId::new());
        let list_submitter_b =
            sample_list_submitter(crate::list_submitters::ListSubmitterId::new());
        list_submitter_a.create(&store).await?;
        list_submitter_b.create(&store).await?;

        let substitute_a = sample_list_submitter(crate::list_submitters::ListSubmitterId::new());
        let substitute_b = sample_list_submitter(crate::list_submitters::ListSubmitterId::new());
        substitute_a.create_substitute(&store).await?;
        substitute_b.create_substitute(&store).await?;

        let mut list = sample_candidate_list(CandidateListId::new());

        list.select_default_submitters(&store)?;

        let chosen_list_submitter = list
            .list_submitter_id
            .expect("default list submitter selected");
        assert!([list_submitter_a.id, list_submitter_b.id].contains(&chosen_list_submitter));

        let chosen_substitutes: BTreeSet<_> =
            list.substitute_list_submitter_ids.iter().copied().collect();
        let expected_substitutes: BTreeSet<_> =
            [substitute_a.id, substitute_b.id].into_iter().collect();
        assert_eq!(expected_substitutes, chosen_substitutes);

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_list_updates_districts() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list = sample_candidate_list(CandidateListId::new());

        list.create(&store).await?;

        let updated_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::DR, ElectoralDistrict::OV],
            ..list.clone()
        };

        updated_list.update_districts(&store).await?;

        assert_eq!(updated_list.id, list.id);
        assert_eq!(
            updated_list.electoral_districts,
            vec![ElectoralDistrict::DR, ElectoralDistrict::OV]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_used_districts() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        // setup
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::OV,
        ]);

        insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        insert_list(&store, vec![ElectoralDistrict::OV]).await?;
        insert_list(&store, vec![]).await?;

        // test
        let result: BTreeSet<ElectoralDistrict> =
            CandidateList::used_districts(&store)?.into_iter().collect();

        // verify
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn get_used_districts_no_lists() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let result = CandidateList::used_districts(&store)?;

        assert_eq!(Vec::<ElectoralDistrict>::new(), result);

        Ok(())
    }

    #[tokio::test]
    async fn get_used_districts_double_districts() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::OV,
        ]);

        // setup
        insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::OV]).await?;

        // test
        let result: BTreeSet<ElectoralDistrict> =
            CandidateList::used_districts(&store)?.into_iter().collect();

        // verify
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn get_used_district_with_exclude() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::GR,
            ElectoralDistrict::OV,
        ]);

        // setup
        insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        insert_list(&store, vec![ElectoralDistrict::GR, ElectoralDistrict::OV]).await?;

        // test
        let result: BTreeSet<ElectoralDistrict> =
            CandidateList::used_districts(&store)?.into_iter().collect();

        // verify
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn test_remove_candidate_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        // setup
        let list_a = sample_candidate_list(CandidateListId::new());
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let list_b = sample_candidate_list(CandidateListId::new());
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list_a.create(&store).await?;
        person_a.create(&store).await?;
        list_a.clone().update_order(&store, &[person_a.id]).await?;

        list_b.create(&store).await?;
        person_b.create(&store).await?;
        list_b.clone().update_order(&store, &[person_b.id]).await?;

        list_a.delete(&store).await?;

        let lists = CandidateListSummary::list(&store)?;
        let list_b_from_db = FullCandidateList::get(&store, list_b.id).unwrap();

        assert_eq!(1, lists.len());
        assert_eq!(list_b.id, lists[0].list.id);
        assert_eq!(1, lists[0].person_count);
        assert_eq!(person_b.id, list_b_from_db.candidates[0].person.id);
        assert_eq!(0, lists[0].duplicate_districts.len());

        Ok(())
    }

    #[tokio::test]
    async fn get_candidate_list_includes_candidates() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        person_a.create(&store).await?;
        person_b.create(&store).await?;
        list.clone()
            .update_order(&store, &[person_a.id, person_b.id])
            .await?;

        let detail = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(2, detail.candidates.len());
        assert_eq!(person_a.id, detail.candidates[0].person.id);
        assert_eq!(person_b.id, detail.candidates[1].person.id);

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_list_order_returns_not_found() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let mut missing_list = sample_candidate_list(CandidateListId::new());
        let err = missing_list.update_order(&store, &[]).await.unwrap_err();
        assert!(matches!(err, AppError::GenericNotFound));

        Ok(())
    }

    #[tokio::test]
    async fn get_full_candidate_list_returns_none_for_missing_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let missing = FullCandidateList::get(&store, CandidateListId::new());
        assert!(missing.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_append_candidate_to_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        person_a.create(&store).await?;
        person_b.create(&store).await?;

        list.append_candidate(&store, person_a.id).await?;
        list.append_candidate(&store, person_b.id).await?;

        let detail = FullCandidateList::get(&store, list_id).expect("candidate list");

        assert_eq!(detail.candidates.len(), 2);
        assert_eq!(detail.candidates[0].person.id, person_a.id);
        assert_eq!(detail.candidates[0].position, 1);
        assert_eq!(detail.candidates[1].person.id, person_b.id);
        assert_eq!(detail.candidates[1].position, 2);

        Ok(())
    }

    #[tokio::test]
    async fn append_candidate_to_list_returns_not_found() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let mut missing_list = sample_candidate_list(CandidateListId::new());
        let err = missing_list
            .append_candidate(&store, PersonId::new())
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::GenericNotFound));

        Ok(())
    }

    #[tokio::test]
    async fn remove_candidate_removes_from_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        person_a.create(&store).await?;
        person_b.create(&store).await?;
        let mut list = store.get_candidate_list(list_id)?;
        list.append_candidate(&store, person_a.id).await?;
        list.append_candidate(&store, person_b.id).await?;

        person_a.delete(&store).await?;

        let detail = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(detail.candidates.len(), 1);
        assert_eq!(detail.candidates[0].person.id, person_b.id);

        Ok(())
    }

    #[tokio::test]
    async fn get_candidate_returns_candidate() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Jansen");

        list.create(&store).await?;
        person.create(&store).await?;
        list.append_candidate(&store, person.id).await?;

        let candidate = store
            .get_candidate_list(list_id)?
            .get_candidate(&store, person.id)
            .await?;

        assert_eq!(candidate.list_id, list_id);
        assert_eq!(candidate.position, 1);
        assert_eq!(candidate.person.id, person.id);

        Ok(())
    }
}
