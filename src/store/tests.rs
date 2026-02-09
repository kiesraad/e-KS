use super::{AppEvent, AppStore, AppStoreData};
use crate::{
    CountryCode, Date, DutchAddress, ElectoralDistrict, FirstName, FullName, HouseNumber,
    HouseNumberAddition, Initials, LastName, LastNamePrefix, Locality, PlaceOfResidence,
    PostalCode, StreetName, UtcDateTime,
    candidate_lists::CandidateListId,
    list_submitters::ListSubmitterId,
    persons::{Gender, PersonId, PersonalInfo, Representative},
    substitute_list_submitters::SubstituteSubmitterId,
    test_utils::{sample_authorised_agent, sample_candidate_list, sample_person},
};
use chrono::{Duration, Utc};

#[test]
fn apply_update_personal_info_replaces_fields() {
    let mut data = AppStoreData::default();
    let person_id = PersonId::new();
    let person = sample_person(person_id);
    data.persons.insert(person_id, person);

    let updated_at = UtcDateTime::from(Utc::now() - Duration::seconds(30));
    let personal_info = PersonalInfo {
        person_id,
        name: FullName {
            last_name: "Smit".parse::<LastName>().expect("last name"),
            last_name_prefix: Some("van".parse::<LastNamePrefix>().expect("prefix")),
            initials: "A.B.".parse::<Initials>().expect("initials"),
        },
        first_name: Some("Anne".parse::<FirstName>().expect("first name")),
        gender: Some(Gender::Male),
        bsn: None,
        no_bsn_confirmed: true,
        date_of_birth: Some("03-04-1988".parse::<Date>().expect("date")),
        place_of_residence: Some(
            "Utrecht"
                .parse::<PlaceOfResidence>()
                .expect("place of residence"),
        ),
        country_of_residence: Some("BE".parse::<CountryCode>().expect("country code")),
        updated_at,
    };

    AppStore::apply(
        AppEvent::UpdatePersonPersonalInfo(personal_info.clone()),
        &mut data,
    );

    let updated = data.persons.get(&person_id).expect("person exists");
    assert_eq!(updated.name.last_name.to_string(), "Smit");
    assert_eq!(
        updated
            .name
            .last_name_prefix
            .as_ref()
            .map(|v| v.to_string()),
        Some("van".to_string())
    );
    assert_eq!(
        updated.first_name.as_ref().map(|v| v.to_string()),
        Some("Anne".to_string())
    );
    assert_eq!(updated.gender, Some(Gender::Male));
    assert_eq!(updated.updated_at, personal_info.updated_at);
}

#[test]
fn apply_update_person_address_and_representative() {
    let mut data = AppStoreData::default();
    let person_id = PersonId::new();
    let person = sample_person(person_id);
    data.persons.insert(person_id, person);

    let address_updated_at = UtcDateTime::from(Utc::now() - Duration::seconds(20));
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

    AppStore::apply(
        AppEvent::UpdatePersonAddress {
            person_id,
            address: new_address.clone(),
            updated_at: address_updated_at,
        },
        &mut data,
    );

    let updated = data.persons.get(&person_id).expect("person exists");
    assert_eq!(updated.address.postal_code, new_address.postal_code);
    assert_eq!(updated.updated_at, address_updated_at);
    assert_eq!(
        updated.representative.name.initials,
        original_representative.name.initials
    );

    let rep_updated_at = UtcDateTime::from(Utc::now() - Duration::seconds(10));
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

    AppStore::apply(
        AppEvent::UpdatePersonRepresentative {
            person_id,
            representative: representative.clone(),
            updated_at: rep_updated_at,
        },
        &mut data,
    );

    let updated = data.persons.get(&person_id).expect("person exists");
    assert_eq!(updated.representative.name.last_name.to_string(), "Bakker");
    assert_eq!(
        updated.representative.address.street_name,
        representative.address.street_name
    );
    assert_eq!(updated.updated_at, rep_updated_at);
}

#[test]
fn apply_add_candidate_to_list_deduplicates() {
    let mut data = AppStoreData::default();
    let list_id = CandidateListId::new();
    let mut list = sample_candidate_list(list_id);
    list.updated_at = UtcDateTime::from(Utc::now() - Duration::seconds(60));

    AppStore::apply(AppEvent::CreateCandidateList(list.clone()), &mut data);

    let person_id = PersonId::new();
    let added_at = UtcDateTime::from(Utc::now() - Duration::seconds(30));
    AppStore::apply(
        AppEvent::AddCandidateToCandidateList {
            list_id,
            person_id,
            updated_at: added_at,
        },
        &mut data,
    );

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.candidates, vec![person_id]);
    assert_eq!(updated.updated_at, added_at);

    let ignored_at = UtcDateTime::from(Utc::now() - Duration::seconds(5));
    AppStore::apply(
        AppEvent::AddCandidateToCandidateList {
            list_id,
            person_id,
            updated_at: ignored_at,
        },
        &mut data,
    );

    let updated_again = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated_again.candidates, vec![person_id]);
    assert_eq!(updated_again.updated_at, added_at);
}

#[test]
fn apply_remove_candidate_from_all_lists_updates_only_matches() {
    let mut data = AppStoreData::default();
    let person_id = PersonId::new();
    let base_time = Utc::now();

    let list_id_with = CandidateListId::new();
    let mut list_with = sample_candidate_list(list_id_with);
    list_with.candidates = vec![person_id];
    list_with.updated_at = UtcDateTime::from(base_time - Duration::seconds(50));

    let list_id_without = CandidateListId::new();
    let mut list_without = sample_candidate_list(list_id_without);
    list_without.updated_at = UtcDateTime::from(base_time - Duration::seconds(40));

    AppStore::apply(AppEvent::CreateCandidateList(list_with), &mut data);
    AppStore::apply(AppEvent::CreateCandidateList(list_without), &mut data);

    let removed_at = UtcDateTime::from(base_time - Duration::seconds(10));
    AppStore::apply(
        AppEvent::RemoveCandidateFromAllCandidateLists {
            person_id,
            updated_at: removed_at,
        },
        &mut data,
    );

    let updated_with = data
        .candidate_lists
        .get(&list_id_with)
        .expect("list with person exists");
    assert!(updated_with.candidates.is_empty());
    assert_eq!(updated_with.updated_at, removed_at);

    let updated_without = data
        .candidate_lists
        .get(&list_id_without)
        .expect("list without person exists");
    assert!(updated_without.candidates.is_empty());
    assert_eq!(
        updated_without.updated_at,
        UtcDateTime::from(base_time - Duration::seconds(40))
    );
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
    list.updated_at = UtcDateTime::from(base_time - Duration::seconds(45));

    AppStore::apply(AppEvent::CreateCandidateList(list), &mut data);

    let removed_at = UtcDateTime::from(base_time - Duration::seconds(5));
    AppStore::apply(
        AppEvent::RemoveCandidateFromCandidateList {
            list_id,
            person_id,
            updated_at: removed_at,
        },
        &mut data,
    );

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.candidates, vec![other_person_id]);
    assert_eq!(updated.updated_at, removed_at);
}

#[test]
fn apply_update_candidate_list_districts_replaces_districts() {
    let mut data = AppStoreData::default();
    let list_id = CandidateListId::new();
    let base_time = Utc::now();

    let mut list = sample_candidate_list(list_id);
    list.electoral_districts = vec![ElectoralDistrict::UT];
    list.updated_at = UtcDateTime::from(base_time - Duration::seconds(50));

    AppStore::apply(AppEvent::CreateCandidateList(list), &mut data);

    let updated_at = UtcDateTime::from(base_time - Duration::seconds(15));
    let districts = vec![ElectoralDistrict::NH, ElectoralDistrict::ZH];
    AppStore::apply(
        AppEvent::UpdateCandidateListDistricts {
            list_id,
            electoral_districts: districts.clone(),
            updated_at,
        },
        &mut data,
    );

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.electoral_districts, districts);
    assert_eq!(updated.updated_at, updated_at);
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
    list.updated_at = UtcDateTime::from(base_time - Duration::seconds(40));

    AppStore::apply(AppEvent::CreateCandidateList(list), &mut data);

    let updated_at = UtcDateTime::from(base_time - Duration::seconds(10));
    let new_order = vec![other_person_id, person_id];
    AppStore::apply(
        AppEvent::UpdateCandidateListOrder {
            list_id,
            candidates: new_order.clone(),
            updated_at,
        },
        &mut data,
    );

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.candidates, new_order);
    assert_eq!(updated.updated_at, updated_at);
}

#[test]
fn apply_update_candidate_list_submitters_sets_ids() {
    let mut data = AppStoreData::default();
    let list_id = CandidateListId::new();
    let mut list = sample_candidate_list(list_id);
    list.list_submitter_id = None;
    list.substitute_list_submitter_ids = Vec::new();

    AppStore::apply(AppEvent::CreateCandidateList(list), &mut data);

    let list_submitter_id = Some(ListSubmitterId::new());
    let substitute_ids = vec![SubstituteSubmitterId::new(), SubstituteSubmitterId::new()];
    let updated_at = UtcDateTime::from(Utc::now() - Duration::seconds(15));

    AppStore::apply(
        AppEvent::UpdateCandidateListSubmitters {
            list_id,
            list_submitter_id,
            substitute_list_submitter_ids: substitute_ids.clone(),
            updated_at,
        },
        &mut data,
    );

    let updated = data.candidate_lists.get(&list_id).expect("list exists");
    assert_eq!(updated.list_submitter_id, list_submitter_id);
    assert_eq!(updated.substitute_list_submitter_ids, substitute_ids);
    assert_eq!(updated.updated_at, updated_at);
}

#[tokio::test]
async fn store_update_applies_event_in_memory() -> Result<(), crate::AppError> {
    let store = AppStore::new_for_test().await;
    let agent_id = crate::authorised_agents::AuthorisedAgentId::new();
    let agent = sample_authorised_agent(agent_id);

    store
        .update(AppEvent::CreateAuthorisedAgent(agent.clone()))
        .await?;

    let loaded = store.get_authorised_agent(agent_id)?;
    assert_eq!(loaded.id, agent.id);

    Ok(())
}
