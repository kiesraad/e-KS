use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AppError, AppEvent, AppStore, ElectionConfig, ElectoralDistrict, UtcDateTime,
    candidate_lists::FullCandidateList,
    candidates::Candidate,
    id_newtype,
    list_submitters::ListSubmitterId,
    persons::{Person, PersonId},
    substitute_list_submitters::SubstituteSubmitterId,
};
use serde::{Deserialize, Serialize};

id_newtype!(pub struct CandidateListId);

#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CandidateList {
    pub id: CandidateListId,
    pub electoral_districts: Vec<ElectoralDistrict>,
    pub candidates: Vec<PersonId>,
    pub list_submitter_id: Option<ListSubmitterId>,
    pub substitute_list_submitter_ids: Vec<SubstituteSubmitterId>,
    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

impl CandidateList {
    pub fn districts_name(&self) -> String {
        self.electoral_districts
            .iter()
            .map(|d| d.title())
            .collect::<Vec<&str>>()
            .join(", ")
    }

    pub fn contains_all_districts(&self, election: &ElectionConfig) -> bool {
        self.electoral_districts.len() == election.electoral_districts().len()
    }

    pub fn used_districts(
        store: &AppStore,
        exclude_list_ids: Vec<CandidateListId>,
    ) -> Result<Vec<ElectoralDistrict>, AppError> {
        let exclude: BTreeSet<CandidateListId> = exclude_list_ids.into_iter().collect();
        let mut used = BTreeSet::<ElectoralDistrict>::new();

        for list in store.get_candidate_lists()? {
            if exclude.contains(&list.id) {
                continue;
            }

            for district in list.electoral_districts {
                used.insert(district);
            }
        }

        Ok(used.into_iter().collect())
    }

    pub async fn update_order(
        store: &AppStore,
        list_id: CandidateListId,
        person_ids: &[PersonId],
    ) -> Result<FullCandidateList, AppError> {
        let mut list = store.get_candidate_list(list_id)?;

        CandidateList::ensure_persons_exist(store, person_ids)?;

        list.candidates = person_ids.to_vec();
        list.updated_at = UtcDateTime::now();

        store
            .update(AppEvent::UpdateCandidateList(list.clone()))
            .await?;

        CandidateList::build_full_candidate_list(store, list)
    }

    pub async fn append_candidate(
        store: &AppStore,
        list_id: CandidateListId,
        person_id: PersonId,
    ) -> Result<(), AppError> {
        let mut list = store.get_candidate_list(list_id)?;

        CandidateList::ensure_persons_exist(store, &[person_id])?;

        if !list.candidates.contains(&person_id) {
            list.candidates.push(person_id);
            list.updated_at = UtcDateTime::now();
            store.update(AppEvent::UpdateCandidateList(list)).await?;
        }

        Ok(())
    }

    pub async fn remove_candidate_from_all(
        store: &AppStore,
        person_id: PersonId,
    ) -> Result<(), AppError> {
        let lists = store.get_candidate_lists()?;

        for mut list in lists {
            if list.candidates.contains(&person_id) {
                list.candidates.retain(|id| *id != person_id);
                list.updated_at = UtcDateTime::now();
                store.update(AppEvent::UpdateCandidateList(list)).await?;
            }
        }

        Ok(())
    }

    pub async fn remove_candidate_from_list(
        store: &AppStore,
        list_id: CandidateListId,
        person_id: PersonId,
    ) -> Result<(), AppError> {
        let mut list = store.get_candidate_list(list_id)?;

        if !list.candidates.contains(&person_id) {
            return Err(AppError::NotFound(
                "candidate not found on list".to_string(),
            ));
        }

        list.candidates.retain(|id| *id != person_id);
        list.updated_at = UtcDateTime::now();
        store.update(AppEvent::UpdateCandidateList(list)).await?;

        Ok(())
    }

    pub async fn get_candidate(
        store: &AppStore,
        list_id: CandidateListId,
        person_id: PersonId,
    ) -> Result<Candidate, AppError> {
        let list = store.get_candidate_list(list_id)?;

        let position = list
            .candidates
            .iter()
            .position(|id| *id == person_id)
            .map(|index| index + 1)
            .ok_or_else(|| AppError::GenericNotFound)?;

        let person = store.get_person(person_id)?;

        Ok(Candidate {
            list_id,
            position,
            person,
        })
    }

    pub fn persons_not_on_list(
        store: &AppStore,
        list_id: CandidateListId,
    ) -> Result<Vec<Person>, AppError> {
        let list = store.get_candidate_list(list_id)?;

        let existing: BTreeMap<PersonId, ()> =
            list.candidates.into_iter().map(|id| (id, ())).collect();

        Ok(store
            .get_persons()?
            .into_iter()
            .filter(|person| !existing.contains_key(&person.id))
            .collect())
    }

    pub async fn create(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::CreateCandidateList(self.clone()))
            .await
    }

    pub async fn update(&self, store: &AppStore) -> Result<(), AppError> {
        store
            .update(AppEvent::UpdateCandidateList(self.clone()))
            .await
    }

    pub async fn delete_by_id(store: &AppStore, list_id: CandidateListId) -> Result<(), AppError> {
        store.update(AppEvent::DeleteCandidateList(list_id)).await
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

    fn ensure_persons_exist(store: &AppStore, person_ids: &[PersonId]) -> Result<(), AppError> {
        let existing: BTreeMap<PersonId, ()> = store
            .get_persons()?
            .into_iter()
            .map(|person| (person.id, ()))
            .collect();

        if person_ids.iter().any(|id| !existing.contains_key(id)) {
            return Err(AppError::GenericNotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AppEvent, AppStore,
        candidate_lists::CandidateListSummary,
        persons::PersonId,
        test_utils::{sample_candidate_list, sample_person_with_last_name},
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

        assert_eq!(list.districts_name(), "Utrecht, Noord-Holland, Drenthe");
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
        let store = AppStore::new_for_test().await;
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
        let store = AppStore::new_for_test().await;
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
        let store = AppStore::new_for_test().await;
        let list_early = CandidateList {
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: UtcDateTime::now(),
            ..Default::default()
        };
        list_early.create(&store).await?;

        // sleep for a second to ensure a different created_at timestamp for the next list
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let list_late = CandidateList {
            electoral_districts: vec![ElectoralDistrict::OV],
            created_at: UtcDateTime::now(),
            ..Default::default()
        };
        list_late.create(&store).await?;

        let lists = store.get_candidate_lists()?;
        assert_eq!(lists.len(), 2);
        assert_eq!(lists[0].id, list_early.id);
        assert_eq!(lists[1].id, list_late.id);

        Ok(())
    }

    #[tokio::test]
    async fn get_candidate_list_returns_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list = sample_candidate_list(CandidateListId::new());

        list.create(&store).await?;

        let loaded = store.get_candidate_list(list.id)?;

        assert_eq!(loaded.id, list.id);

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_list_updates_districts() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list = sample_candidate_list(CandidateListId::new());

        list.create(&store).await?;

        let updated_list = CandidateList {
            electoral_districts: vec![ElectoralDistrict::DR, ElectoralDistrict::OV],
            ..list.clone()
        };

        updated_list.update(&store).await?;

        assert_eq!(updated_list.id, list.id);
        assert_eq!(
            updated_list.electoral_districts,
            vec![ElectoralDistrict::DR, ElectoralDistrict::OV]
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_used_districts() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
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
        let result: BTreeSet<ElectoralDistrict> = CandidateList::used_districts(&store, vec![])?
            .into_iter()
            .collect();

        // verify
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn get_used_districts_no_lists() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let result = CandidateList::used_districts(&store, vec![])?;

        assert_eq!(Vec::<ElectoralDistrict>::new(), result);

        Ok(())
    }

    #[tokio::test]
    async fn get_used_districts_double_districts() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::OV,
        ]);

        // setup
        insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::OV]).await?;

        // test
        let result: BTreeSet<ElectoralDistrict> = CandidateList::used_districts(&store, vec![])?
            .into_iter()
            .collect();

        // verify
        assert_eq!(expected, result);
        Ok(())
    }

    #[tokio::test]
    async fn get_used_district_with_exclude() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::GR,
            ElectoralDistrict::OV,
        ]);

        // setup
        insert_list(&store, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        insert_list(&store, vec![ElectoralDistrict::GR, ElectoralDistrict::OV]).await?;

        let exclude_id = insert_list(&store, vec![ElectoralDistrict::GR, ElectoralDistrict::LI])
            .await?
            .id;

        // test
        let result: BTreeSet<ElectoralDistrict> =
            CandidateList::used_districts(&store, vec![exclude_id])?
                .into_iter()
                .collect();

        // verify
        assert_eq!(expected, result);

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_candidate_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        // setup
        let list_a = sample_candidate_list(CandidateListId::new());
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let list_b = sample_candidate_list(CandidateListId::new());
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list_a.create(&store).await?;
        store
            .update(AppEvent::CreatePerson(person_a.clone()))
            .await?;
        CandidateList::update_order(&store, list_a.id, &[person_a.id]).await?;

        list_b.create(&store).await?;
        store
            .update(AppEvent::CreatePerson(person_b.clone()))
            .await?;
        CandidateList::update_order(&store, list_b.id, &[person_b.id]).await?;

        // test
        CandidateList::delete_by_id(&store, list_a.id).await?;

        // verify
        let lists = CandidateListSummary::list(&store)?;
        let list_b_from_db = FullCandidateList::get(&store, list_b.id).unwrap();
        // one list remains
        assert_eq!(1, lists.len());
        // the correct list got deleted
        assert_eq!(list_b.id, lists[0].list.id);
        // and only persons got removed associated with the deleted list
        assert_eq!(1, lists[0].person_count);
        assert_eq!(person_b.id, list_b_from_db.candidates[0].person.id);
        // no duplicate districts
        assert_eq!(0, lists[0].duplicate_districts.len());

        Ok(())
    }

    #[tokio::test]
    async fn get_candidate_list_includes_candidates() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        store
            .update(AppEvent::CreatePerson(person_a.clone()))
            .await?;
        store
            .update(AppEvent::CreatePerson(person_b.clone()))
            .await?;
        CandidateList::update_order(&store, list_id, &[person_a.id, person_b.id]).await?;

        let detail = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(2, detail.candidates.len());
        assert_eq!(person_a.id, detail.candidates[0].person.id);
        assert_eq!(person_b.id, detail.candidates[1].person.id);

        Ok(())
    }

    #[tokio::test]
    async fn update_candidate_list_order_returns_not_found() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let err = CandidateList::update_order(&store, CandidateListId::new(), &[])
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::GenericNotFound));

        Ok(())
    }

    #[tokio::test]
    async fn get_full_candidate_list_returns_none_for_missing_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let missing = FullCandidateList::get(&store, CandidateListId::new());
        assert!(missing.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_append_candidate_to_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        store
            .update(AppEvent::CreatePerson(person_a.clone()))
            .await?;
        store
            .update(AppEvent::CreatePerson(person_b.clone()))
            .await?;

        CandidateList::append_candidate(&store, list_id, person_a.id).await?;
        CandidateList::append_candidate(&store, list_id, person_b.id).await?;

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
        let store = AppStore::new_for_test().await;
        let err = CandidateList::append_candidate(&store, CandidateListId::new(), PersonId::new())
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::GenericNotFound));

        Ok(())
    }

    #[tokio::test]
    async fn remove_candidate_removes_from_list() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        list.create(&store).await?;
        store
            .update(AppEvent::CreatePerson(person_a.clone()))
            .await?;
        store
            .update(AppEvent::CreatePerson(person_b.clone()))
            .await?;
        CandidateList::append_candidate(&store, list_id, person_a.id).await?;
        CandidateList::append_candidate(&store, list_id, person_b.id).await?;

        CandidateList::remove_candidate_from_all(&store, person_a.id).await?;

        let detail = FullCandidateList::get(&store, list_id).expect("candidate list");
        assert_eq!(detail.candidates.len(), 1);
        assert_eq!(detail.candidates[0].person.id, person_b.id);

        Ok(())
    }

    #[tokio::test]
    async fn get_candidate_returns_candidate() -> Result<(), AppError> {
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Jansen");

        list.create(&store).await?;
        store.update(AppEvent::CreatePerson(person.clone())).await?;
        CandidateList::append_candidate(&store, list_id, person.id).await?;

        let candidate = CandidateList::get_candidate(&store, list_id, person.id).await?;
        assert_eq!(candidate.list_id, list_id);
        assert_eq!(candidate.position, 1);
        assert_eq!(candidate.person.id, person.id);

        Ok(())
    }
}
