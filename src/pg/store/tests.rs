use crate::{
    AppError, ElectoralDistrict, PgEvent, PgStore, PgStoreData,
    candidate_lists::CandidateListId,
    common::{
        DutchAddress, FullName, HouseNumber, HouseNumberAddition, Initials, LastName, Locality,
        PostalCode, StreetName, UtcDateTime,
    },
    persons::{PersonId, Representative},
    store::{StoreData, StoreEvent},
    test_utils::{sample_candidate_list, sample_name_authorisation, sample_person},
};
use chrono::{Duration, Utc};

#[test]
fn apply_update_person_address_and_representative() {
    let mut data = PgStoreData::default();
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
        known_in_bag: Some(true),
    };

    let original_representative = data
        .persons
        .get(&person_id)
        .expect("person exists")
        .representative
        .clone();

    data.apply(StoreEvent::new_at(
        1,
        PgEvent::UpdatePersonAddress {
            person_id,
            address: new_address.clone(),
        },
        address_event_time,
    ));

    let updated = data.persons.get(&person_id).expect("person exists");
    assert_eq!(updated.address.postal_code, new_address.postal_code);
    assert_eq!(updated.updated_at, UtcDateTime::from(address_event_time));
    assert_eq!(updated.representative, original_representative);

    let rep_event_time = Utc::now() - Duration::seconds(10);
    let representative = Representative {
        name: FullName {
            first_name: None,
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
            known_in_bag: Some(true),
        },
    };

    data.apply(StoreEvent::new_at(
        2,
        PgEvent::UpdatePersonRepresentative {
            person_id,
            representative: Some(representative.clone()),
        },
        rep_event_time,
    ));

    let updated = data.persons.get(&person_id).expect("person exists");
    assert_eq!(
        updated
            .representative
            .as_ref()
            .unwrap()
            .name
            .last_name
            .to_string(),
        "Bakker"
    );
    assert_eq!(
        updated.representative.as_ref().unwrap().address.street_name,
        representative.address.street_name
    );
    assert_eq!(updated.updated_at, UtcDateTime::from(rep_event_time));
}

#[test]
fn apply_add_candidate_to_list_deduplicates() {
    let mut data = PgStoreData::default();
    let list_id = CandidateListId::new();
    let list = sample_candidate_list(list_id);

    let created_at = Utc::now() - Duration::seconds(60);
    data.apply(StoreEvent::new_at(
        1,
        PgEvent::CreateCandidateList(list.clone()),
        created_at,
    ));

    let person_id = PersonId::new();
    let added_at = Utc::now() - Duration::seconds(30);
    data.apply(StoreEvent::new_at(
        2,
        PgEvent::AddCandidateToCandidateList { list_id, person_id },
        added_at,
    ));

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.candidates, vec![person_id]);

    let ignored_at = Utc::now() - Duration::seconds(5);
    data.apply(StoreEvent::new_at(
        3,
        PgEvent::AddCandidateToCandidateList { list_id, person_id },
        ignored_at,
    ));

    let updated_again = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated_again.candidates, vec![person_id]);
}

#[test]
fn apply_delete_person_updates_only_candidate_lists_with_that_candidate() {
    let mut data = PgStoreData::default();
    let person_id = PersonId::new();
    let base_time = Utc::now();

    let list_id_with = CandidateListId::new();
    let mut list_with = sample_candidate_list(list_id_with);
    list_with.candidates = vec![person_id];

    let list_id_without = CandidateListId::new();
    let list_without = sample_candidate_list(list_id_without);

    data.apply(StoreEvent::new_at(
        1,
        PgEvent::CreateCandidateList(list_with),
        base_time - Duration::seconds(50),
    ));
    data.apply(StoreEvent::new_at(
        2,
        PgEvent::CreateCandidateList(list_without),
        base_time - Duration::seconds(40),
    ));

    let removed_at = base_time - Duration::seconds(10);
    data.apply(StoreEvent::new_at(
        3,
        PgEvent::DeletePerson { person_id },
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
    let mut data = PgStoreData::default();
    let list_id = CandidateListId::new();
    let person_id = PersonId::new();
    let other_person_id = PersonId::new();
    let base_time = Utc::now();

    let mut list = sample_candidate_list(list_id);
    list.candidates = vec![person_id, other_person_id];

    data.apply(StoreEvent::new_at(
        1,
        PgEvent::CreateCandidateList(list),
        base_time - Duration::seconds(45),
    ));

    let removed_at = base_time - Duration::seconds(5);
    data.apply(StoreEvent::new_at(
        2,
        PgEvent::RemoveCandidateFromCandidateList { list_id, person_id },
        removed_at,
    ));

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.candidates, vec![other_person_id]);
}

#[test]
fn apply_update_candidate_list_districts_replaces_districts() {
    let mut data = PgStoreData::default();
    let list_id = CandidateListId::new();
    let base_time = Utc::now();

    let mut list = sample_candidate_list(list_id);
    list.electoral_districts = vec![ElectoralDistrict::UT];

    data.apply(StoreEvent::new_at(
        1,
        PgEvent::CreateCandidateList(list),
        base_time - Duration::seconds(50),
    ));

    let updated_at = base_time - Duration::seconds(15);
    let districts = vec![ElectoralDistrict::NH, ElectoralDistrict::ZH];
    data.apply(StoreEvent::new_at(
        2,
        PgEvent::UpdateCandidateListDistricts {
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
    let mut data = PgStoreData::default();
    let list_id = CandidateListId::new();
    let person_id = PersonId::new();
    let other_person_id = PersonId::new();
    let base_time = Utc::now();

    let mut list = sample_candidate_list(list_id);
    list.candidates = vec![person_id, other_person_id];

    data.apply(StoreEvent::new_at(
        1,
        PgEvent::CreateCandidateList(list),
        base_time - Duration::seconds(40),
    ));

    let updated_at = base_time - Duration::seconds(10);
    let new_order = vec![other_person_id, person_id];
    data.apply(StoreEvent::new_at(
        2,
        PgEvent::UpdateCandidateListOrder {
            list_id,
            candidates: new_order.clone(),
        },
        updated_at,
    ));

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.candidates, new_order);
}

#[tokio::test]
async fn store_update_applies_event_in_memory() -> Result<(), AppError> {
    let store = PgStore::new_for_test();
    let id = crate::name_authorisations::NameAuthorisationId::new();
    let name_authorisation = sample_name_authorisation(id);

    name_authorisation.create(&store).await?;

    let loaded = store.get_name_authorisation(id)?;
    assert_eq!(loaded.id, name_authorisation.id);

    Ok(())
}

#[test]
fn snapshot_until_replays_up_to_the_target_event_and_drops_the_log() {
    let person_a = PersonId::new();
    let person_b = PersonId::new();
    let events = vec![
        StoreEvent::new(1, PgEvent::CreatePerson(sample_person(person_a))),
        StoreEvent::new(2, PgEvent::CreatePerson(sample_person(person_b))),
        StoreEvent::new(
            3,
            PgEvent::DeletePerson {
                person_id: person_a,
            },
        ),
    ];

    // Up to event 2: both persons present, and the snapshot carries no event log.
    let at_two = PgStoreData::snapshot_until(&events, 2);
    assert!(at_two.persons.contains_key(&person_a));
    assert!(at_two.persons.contains_key(&person_b));
    assert!(at_two.events.is_empty());

    // Up to event 3: the deletion has been applied.
    let at_three = PgStoreData::snapshot_until(&events, 3);
    assert!(!at_three.persons.contains_key(&person_a));
    assert!(at_three.persons.contains_key(&person_b));
    assert!(at_three.events.is_empty());
}

#[test]
fn snapshot_until_ignores_events_past_the_target() {
    let person_a = PersonId::new();
    let person_b = PersonId::new();
    let events = vec![
        StoreEvent::new(1, PgEvent::CreatePerson(sample_person(person_a))),
        StoreEvent::new(2, PgEvent::CreatePerson(sample_person(person_b))),
    ];

    // Stopping at event 1 leaves the later creation out of the snapshot.
    let snapshot = PgStoreData::snapshot_until(&events, 1);
    assert!(snapshot.persons.contains_key(&person_a));
    assert!(!snapshot.persons.contains_key(&person_b));
}

/// In paper-corrections mode the handle reads the CSB stream's corrected
/// projection, and every dispatched app event is wrapped in
/// [`crate::CsbEvent::PaperCorrectedUpdate`] and persisted on that stream.
#[tokio::test]
async fn paper_corrections_store_wraps_events_and_refreshes_its_snapshot() -> Result<(), AppError> {
    use crate::{CsbEvent, CsbStore, test_utils::sample_political_group};

    let csb_store = CsbStore::new_for_test();
    csb_store.set_political_group(sample_political_group());
    let store = PgStore::paper_corrections(csb_store.clone());

    // Reads serve a snapshot of the corrected projection.
    assert_eq!(
        store.get_political_group().display_name,
        sample_political_group().display_name
    );

    let mut corrected_group = sample_political_group();
    corrected_group.display_name = Some("Gecorrigeerde Naam".parse().unwrap());
    store
        .update(PgEvent::UpdatePoliticalGroup(corrected_group.clone()))
        .await?;

    // The event lands on the CSB stream, wrapped as a paper correction.
    {
        let data = csb_store.data.read();
        assert!(matches!(
            &data.events.last().unwrap().payload,
            CsbEvent::PaperCorrectedUpdate(inner)
                if matches!(**inner, PgEvent::UpdatePoliticalGroup(_))
        ));
        assert_eq!(
            data.paper_corrected_data.political_group.display_name,
            corrected_group.display_name
        );
        // The imported snapshot stays untouched.
        assert_eq!(
            data.imported_data.political_group.display_name,
            sample_political_group().display_name
        );
    }

    // The request-local snapshot observes the correction right away.
    assert_eq!(
        store.get_political_group().display_name,
        corrected_group.display_name
    );

    Ok(())
}

#[cfg(feature = "database")]
mod database_tests {
    use super::*;
    use crate::{
        ElectionConfig, Province, Scope, StreamId, persons::PersonId, test_utils::sample_person,
    };
    use chrono::Utc;
    use sqlx::PgPool;

    fn test_encryption() -> crate::store::EventEncryption {
        crate::store::EventEncryption::new(&secrecy::SecretString::from("test-encryption-key"))
    }

    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test(migrations = false)]
    async fn update_persists_and_load_replays(pool: PgPool) -> Result<(), AppError> {
        #[cfg(feature = "migrations")]
        crate::store::database::migrate(&pool).await?;

        let encryption = test_encryption();
        let group_id = StreamId::new();
        let store = PgStore::new_with_pool_for_stream(
            pool.clone(),
            group_id,
            ElectionConfig::EK27,
            &encryption,
        )
        .await
        .unwrap();
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        let loaded = store.get_person(person_id)?;
        assert_eq!(loaded.id, person_id);

        let fresh_store =
            PgStore::new_with_pool_for_stream(pool, group_id, ElectionConfig::EK27, &encryption)
                .await
                .unwrap();
        fresh_store.load().await?;

        let reloaded = fresh_store.get_person(person_id)?;
        assert_eq!(reloaded.id, person_id);

        Ok(())
    }

    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test(migrations = false)]
    async fn load_fails_on_invalid_payloads(pool: PgPool) -> Result<(), AppError> {
        #[cfg(feature = "migrations")]
        crate::store::database::migrate(&pool).await?;

        let encryption = test_encryption();
        let group_id = StreamId::new();
        let store = PgStore::new_with_pool_for_stream(
            pool.clone(),
            group_id,
            ElectionConfig::EK27,
            &encryption,
        )
        .await
        .unwrap();
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        person.create(&store).await?;

        // Insert a bogus event: random payload bytes and a hash that does not
        // match the chain. Either the chain check or the AES-GCM tag will reject it.
        let invalid_payload: Vec<u8> = vec![0u8; 64];
        let invalid_hash: Vec<u8> = vec![0u8; 32];
        let election_id = ElectionConfig::EK27.stable_id();
        sqlx::query(
            r#"INSERT INTO events (stream_id, election, event_id, created_at, hash, payload)
            VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(store.stream_id.uuid())
        .bind(&election_id)
        .bind(2_i64)
        .bind(Utc::now())
        .bind(invalid_hash)
        .bind(invalid_payload)
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"UPDATE streams SET last_event_id = $3
               WHERE stream_id = $1 AND election = $2"#,
        )
        .bind(store.stream_id.uuid())
        .bind(&election_id)
        .bind(2_i64)
        .execute(&pool)
        .await?;

        let fresh_store =
            PgStore::new_with_pool_for_stream(pool, group_id, ElectionConfig::EK27, &encryption)
                .await
                .unwrap();

        let err = fresh_store
            .load()
            .await
            .expect_err("load must fail when an event's payload cannot be decrypted");
        assert!(matches!(err, AppError::EventDecodeError(_)));

        Ok(())
    }

    /// Each stream row carries its scope (set at creation). `streams_by_scope`
    /// lists every data-bearing `(stream_id, election)` of the requested scope,
    /// and a committee stream never leaks into the political-group listing,
    /// even across several elections under one stream_id.
    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test(migrations = false)]
    async fn streams_by_scope_lists_stream_election_pairs_per_scope(
        pool: PgPool,
    ) -> Result<(), AppError> {
        use crate::store::database::{ensure_stream, streams_by_scope};

        #[cfg(feature = "migrations")]
        crate::store::database::migrate(&pool).await?;

        let committee = StreamId::new();
        let group = StreamId::new();
        let ek27 = ElectionConfig::EK27;
        let ps27 = ElectionConfig::PS27(Province::GE);

        // The committee stream joins two elections; each row is created with the
        // committee scope. The political group joins one.
        ensure_stream(&pool, committee, ek27, Scope::CentralElectoralCommittee).await?;
        ensure_stream(&pool, committee, ps27, Scope::CentralElectoralCommittee).await?;
        ensure_stream(&pool, group, ek27, Scope::PoliticalGroup).await?;

        // Empty placeholder rows (last_event_id = 0) are not yet accessible.
        assert!(
            streams_by_scope(&pool, Scope::CentralElectoralCommittee)
                .await?
                .is_empty(),
            "data-less streams are excluded"
        );

        // Give every stream some persisted data so it counts as accessible.
        sqlx::query("UPDATE streams SET last_event_id = 1")
            .execute(&pool)
            .await?;

        // Both committee elections are listed under the committee scope.
        let mut committee_streams =
            streams_by_scope(&pool, Scope::CentralElectoralCommittee).await?;
        committee_streams.sort_by_key(|(_, election)| election.stable_id());
        assert_eq!(
            committee_streams,
            vec![(committee, ek27), (committee, ps27)]
        );

        // The committee stream never leaks into the (default) political-group
        // listing; only the political group's own stream appears there.
        let political = streams_by_scope(&pool, Scope::PoliticalGroup).await?;
        assert_eq!(political, vec![(group, ek27)]);

        Ok(())
    }

    /// A package hash resolves to the political-group event that produced it,
    /// both for a full chain hash and for the 16-byte prefix rendered on
    /// documents; an unrelated prefix resolves to nothing.
    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test(migrations = false)]
    async fn find_event_by_hash_prefix_locates_political_group_events(
        pool: PgPool,
    ) -> Result<(), AppError> {
        use crate::store::database::find_event_by_hash_prefix;

        #[cfg(feature = "migrations")]
        crate::store::database::migrate(&pool).await?;

        let encryption = test_encryption();
        let group = StreamId::new();
        let store = PgStore::new_with_pool_for_stream(
            pool.clone(),
            group,
            ElectionConfig::EK27,
            &encryption,
        )
        .await
        .unwrap();
        sample_person(PersonId::new()).create(&store).await?;

        let target = store
            .get_events()
            .last()
            .cloned()
            .expect("at least one event");
        let expected = Some((group, ElectionConfig::EK27, target.event_id));

        assert_eq!(
            find_event_by_hash_prefix(&pool, &target.hash).await?,
            expected
        );
        assert_eq!(
            find_event_by_hash_prefix(&pool, &target.hash[..16]).await?,
            expected
        );
        assert_eq!(find_event_by_hash_prefix(&pool, &[0xFFu8; 32]).await?, None);

        Ok(())
    }

    /// The lookup is restricted to political-group streams, so a committee
    /// (CSB) event is never returned even when its hash matches exactly.
    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test(migrations = false)]
    async fn find_event_by_hash_prefix_ignores_committee_events(
        pool: PgPool,
    ) -> Result<(), AppError> {
        use crate::store::database::{ensure_stream, find_event_by_hash_prefix};

        #[cfg(feature = "migrations")]
        crate::store::database::migrate(&pool).await?;

        let committee = StreamId::new();
        let election_id = ElectionConfig::EK27.stable_id();
        ensure_stream(
            &pool,
            committee,
            ElectionConfig::EK27,
            Scope::CentralElectoralCommittee,
        )
        .await?;

        let hash = vec![0x42u8; 32];
        sqlx::query(
            r#"INSERT INTO events (stream_id, election, event_id, created_at, hash, payload)
            VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(committee.uuid())
        .bind(&election_id)
        .bind(1_i64)
        .bind(Utc::now())
        .bind(&hash)
        .bind(vec![0u8; 8])
        .execute(&pool)
        .await?;

        assert_eq!(find_event_by_hash_prefix(&pool, &hash).await?, None);

        Ok(())
    }
}
