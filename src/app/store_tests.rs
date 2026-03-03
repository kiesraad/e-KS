use crate::{
    AppError, AppEvent, AppStore, AppStoreData, ElectoralDistrict,
    candidate_lists::CandidateListId,
    common::{
        DutchAddress, FullName, HouseNumber, HouseNumberAddition, Initials, LastName, Locality,
        PostalCode, StreetName, UtcDateTime,
    },
    list_submitters::ListSubmitterId,
    persons::{PersonId, Representative},
    store::{StoreData, StoreEvent},
    substitute_list_submitters::SubstituteSubmitterId,
    test_utils::{sample_authorised_agent, sample_candidate_list, sample_person},
};
use chrono::{Duration, Utc};

#[test]
fn apply_update_person_address_and_representative() {
    let mut data = AppStoreData::default();
    let person_id = PersonId::new();
    let person = sample_person(person_id);
    data.persons.insert(person_id, person);

    let address_event_time = Utc::now() - Duration::seconds(20);
    let new_address = DutchAddress {
        locality: Some("Utrecht".parse::<Locality>().expect("locality")),
        postal_code: Some("3511 AA".parse::<PostalCode>().expect("postal code")),
        house_number: Some("12".parse::<HouseNumber>().expect("house number")),
        house_number_addition: Some(
            "A".parse::<HouseNumberAddition>()
                .expect("house number addition"),
        ),
        street_name: Some("Oudegracht".parse::<StreetName>().expect("street name")),
    };

    let original_representative = data
        .persons
        .get(&person_id)
        .expect("person exists")
        .representative
        .clone();

    data.apply(StoreEvent::new_at(
        1,
        AppEvent::UpdatePersonAddress {
            person_id,
            address: new_address.clone(),
        },
        address_event_time,
    ));

    let updated = data.persons.get(&person_id).expect("person exists");
    assert_eq!(updated.address.postal_code, new_address.postal_code);
    assert_eq!(updated.updated_at, UtcDateTime::from(address_event_time));
    assert_eq!(
        updated.representative.name.initials,
        original_representative.name.initials
    );

    let rep_event_time = Utc::now() - Duration::seconds(10);
    let representative = Representative {
        name: FullName {
            last_name: "Bakker".parse::<LastName>().expect("last name"),
            last_name_prefix: None,
            initials: "C.D.".parse::<Initials>().expect("initials"),
        },
        address: DutchAddress {
            locality: Some("Rotterdam".parse::<Locality>().expect("locality")),
            postal_code: Some("3011 CC".parse::<PostalCode>().expect("postal code")),
            house_number: Some("5".parse::<HouseNumber>().expect("house number")),
            house_number_addition: None,
            street_name: Some("Coolsingel".parse::<StreetName>().expect("street name")),
        },
    };

    data.apply(StoreEvent::new_at(
        2,
        AppEvent::UpdatePersonRepresentative {
            person_id,
            representative: representative.clone(),
        },
        rep_event_time,
    ));

    let updated = data.persons.get(&person_id).expect("person exists");
    assert_eq!(updated.representative.name.last_name.to_string(), "Bakker");
    assert_eq!(
        updated.representative.address.street_name,
        representative.address.street_name
    );
    assert_eq!(updated.updated_at, UtcDateTime::from(rep_event_time));
}

#[test]
fn apply_add_candidate_to_list_deduplicates() {
    let mut data = AppStoreData::default();
    let list_id = CandidateListId::new();
    let list = sample_candidate_list(list_id);

    let created_at = Utc::now() - Duration::seconds(60);
    data.apply(StoreEvent::new_at(
        1,
        AppEvent::CreateCandidateList(list.clone()),
        created_at,
    ));

    let person_id = PersonId::new();
    let added_at = Utc::now() - Duration::seconds(30);
    data.apply(StoreEvent::new_at(
        2,
        AppEvent::AddCandidateToCandidateList { list_id, person_id },
        added_at,
    ));

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.candidates, vec![person_id]);

    let ignored_at = Utc::now() - Duration::seconds(5);
    data.apply(StoreEvent::new_at(
        3,
        AppEvent::AddCandidateToCandidateList { list_id, person_id },
        ignored_at,
    ));

    let updated_again = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated_again.candidates, vec![person_id]);
}

#[test]
fn apply_delete_person_updates_only_candidate_lists_with_that_candidate() {
    let mut data = AppStoreData::default();
    let person_id = PersonId::new();
    let base_time = Utc::now();

    let list_id_with = CandidateListId::new();
    let mut list_with = sample_candidate_list(list_id_with);
    list_with.candidates = vec![person_id];

    let list_id_without = CandidateListId::new();
    let list_without = sample_candidate_list(list_id_without);

    data.apply(StoreEvent::new_at(
        1,
        AppEvent::CreateCandidateList(list_with),
        base_time - Duration::seconds(50),
    ));
    data.apply(StoreEvent::new_at(
        2,
        AppEvent::CreateCandidateList(list_without),
        base_time - Duration::seconds(40),
    ));

    let removed_at = base_time - Duration::seconds(10);
    data.apply(StoreEvent::new_at(
        3,
        AppEvent::DeletePerson { person_id },
        removed_at,
    ));

    let updated_with = data
        .candidate_lists
        .get(&list_id_with)
        .expect("list with person exists");
    assert!(updated_with.candidates.is_empty());

    let updated_without = data
        .candidate_lists
        .get(&list_id_without)
        .expect("list without person exists");
    assert!(updated_without.candidates.is_empty());
}

#[test]
fn apply_remove_candidate_from_candidate_list_updates_list() {
    let mut data = AppStoreData::default();
    let list_id = CandidateListId::new();
    let person_id = PersonId::new();
    let other_person_id = PersonId::new();
    let base_time = Utc::now();

    let mut list = sample_candidate_list(list_id);
    list.candidates = vec![person_id, other_person_id];

    data.apply(StoreEvent::new_at(
        1,
        AppEvent::CreateCandidateList(list),
        base_time - Duration::seconds(45),
    ));

    let removed_at = base_time - Duration::seconds(5);
    data.apply(StoreEvent::new_at(
        2,
        AppEvent::RemoveCandidateFromCandidateList { list_id, person_id },
        removed_at,
    ));

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.candidates, vec![other_person_id]);
}

#[test]
fn apply_update_candidate_list_districts_replaces_districts() {
    let mut data = AppStoreData::default();
    let list_id = CandidateListId::new();
    let base_time = Utc::now();

    let mut list = sample_candidate_list(list_id);
    list.electoral_districts = vec![ElectoralDistrict::UT];

    data.apply(StoreEvent::new_at(
        1,
        AppEvent::CreateCandidateList(list),
        base_time - Duration::seconds(50),
    ));

    let updated_at = base_time - Duration::seconds(15);
    let districts = vec![ElectoralDistrict::NH, ElectoralDistrict::ZH];
    data.apply(StoreEvent::new_at(
        2,
        AppEvent::UpdateCandidateListDistricts {
            list_id,
            electoral_districts: districts.clone(),
        },
        updated_at,
    ));

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.electoral_districts, districts);
}

#[test]
fn apply_update_candidate_list_order_replaces_candidates() {
    let mut data = AppStoreData::default();
    let list_id = CandidateListId::new();
    let person_id = PersonId::new();
    let other_person_id = PersonId::new();
    let base_time = Utc::now();

    let mut list = sample_candidate_list(list_id);
    list.candidates = vec![person_id, other_person_id];

    data.apply(StoreEvent::new_at(
        1,
        AppEvent::CreateCandidateList(list),
        base_time - Duration::seconds(40),
    ));

    let updated_at = base_time - Duration::seconds(10);
    let new_order = vec![other_person_id, person_id];
    data.apply(StoreEvent::new_at(
        2,
        AppEvent::UpdateCandidateListOrder {
            list_id,
            candidates: new_order.clone(),
        },
        updated_at,
    ));

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.candidates, new_order);
}

#[test]
fn apply_update_candidate_list_submitters_sets_ids() {
    let mut data = AppStoreData::default();
    let list_id = CandidateListId::new();
    let mut list = sample_candidate_list(list_id);
    list.list_submitter_id = None;
    list.substitute_list_submitter_ids = Vec::new();

    let created_at = Utc::now() - Duration::seconds(30);
    data.apply(StoreEvent::new_at(
        1,
        AppEvent::CreateCandidateList(list),
        created_at,
    ));

    let list_submitter_id = Some(ListSubmitterId::new());
    let substitute_ids = vec![SubstituteSubmitterId::new(), SubstituteSubmitterId::new()];
    let updated_at = Utc::now() - Duration::seconds(15);

    data.apply(StoreEvent::new_at(
        2,
        AppEvent::UpdateCandidateListSubmitters {
            list_id,
            list_submitter_id,
            substitute_list_submitter_ids: substitute_ids.clone(),
        },
        updated_at,
    ));

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.list_submitter_id, list_submitter_id);
    assert_eq!(updated.substitute_list_submitter_ids, substitute_ids);
}

#[tokio::test]
async fn store_update_applies_event_in_memory() -> Result<(), AppError> {
    let store = AppStore::new_for_test();
    let agent_id = crate::authorised_agents::AuthorisedAgentId::new();
    let agent = sample_authorised_agent(agent_id);

    agent.create(&store).await?;

    let loaded = store.get_authorised_agent(agent_id)?;
    assert_eq!(loaded.id, agent.id);

    Ok(())
}

#[cfg(feature = "database")]
mod database_tests {
    use super::*;
    use crate::{PoliticalGroupId, persons::PersonId, test_utils::sample_person};
    use chrono::Utc;
    use sqlx::PgPool;

    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test(migrations = false)]
    async fn update_persists_and_load_replays(pool: PgPool) -> Result<(), AppError> {
        #[cfg(feature = "migrations")]
        crate::store::database::migrate(&pool).await?;

        let group_id = PoliticalGroupId::new();
        let store = AppStore::new_with_pool_for_stream(pool.clone(), group_id.uuid())
            .await
            .unwrap();
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let loaded = store.get_person(person_id)?;
        assert_eq!(loaded.id, person_id);

        let fresh_store = AppStore::new_with_pool_for_stream(pool, group_id.uuid())
            .await
            .unwrap();
        fresh_store.load().await?;

        let reloaded = fresh_store.get_person(person_id)?;
        assert_eq!(reloaded.id, person_id);

        Ok(())
    }

    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test(migrations = false)]
    async fn load_skips_invalid_payloads(pool: PgPool) -> Result<(), AppError> {
        #[cfg(feature = "migrations")]
        crate::store::database::migrate(&pool).await?;

        let group_id = PoliticalGroupId::new();
        let store = AppStore::new_with_pool_for_stream(pool.clone(), group_id.uuid())
            .await
            .unwrap();
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let invalid_payload = serde_json::json!({"not": "an app event"});
        sqlx::query(
            r#"INSERT INTO events (stream_id, event_id, created_at, payload)
            VALUES ($1, $2, $3, $4)"#,
        )
        .bind(store.stream_id)
        .bind(2_i64)
        .bind(Utc::now())
        .bind(invalid_payload)
        .execute(&pool)
        .await?;

        sqlx::query(r#"UPDATE streams SET last_event_id = $2 WHERE stream_id = $1"#)
            .bind(store.stream_id)
            .bind(2_i64)
            .execute(&pool)
            .await?;

        let fresh_store = AppStore::new_with_pool_for_stream(pool, group_id.uuid())
            .await
            .unwrap();
        fresh_store.load().await?;

        let reloaded = fresh_store.get_person(person_id)?;
        assert_eq!(reloaded.id, person_id);

        Ok(())
    }
}
