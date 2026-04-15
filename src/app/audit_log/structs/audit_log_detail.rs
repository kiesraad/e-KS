//! Detailed (per-field) view of a single audit-log event.
//!
//! `AuditLogDetail::compute` replays the event stream to reconstruct the state
//! before and after the target event, then diffs the flattened JSON
//! representations to produce a list of `FieldChange`s.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::{
    AppEvent, AppStoreData, ElectionConfig, Locale,
    store::{StoreData, StoreEvent},
    trans,
};

use super::{
    audit_log_entry::AuditLogEntry,
    entity_refs::{EntityRef, build_refs_for_key},
    event_payload::extract_old_new,
    json_flatten::flatten,
};

/// Flattened-JSON keys to skip from the diff (metadata, not meaningful changes).
const EXCLUDED_FIELDS: &[&str] = &["id", "updated_at", "created_at"];

/// A single field-level change in an audit log event.
///
/// When the field is an entity ID (or list of IDs), `old_refs` / `new_refs`
/// carry resolved references so the template can render abbreviated clickable
/// links plus a human-readable description. Otherwise the raw `old_value` /
/// `new_value` strings are rendered.
pub struct FieldChange {
    pub field: String,
    pub old_value: String,
    pub new_value: String,
    pub old_refs: Vec<EntityRef>,
    pub new_refs: Vec<EntityRef>,
}

/// Detailed view of an audit log event, including field-level changes.
pub struct AuditLogDetail {
    pub event_id: usize,
    pub description: String,
    pub details: String,
    pub subject_id_full: String,
    pub subject_path: String,
    pub created_at: DateTime<Utc>,
    pub changes: Vec<FieldChange>,
}

impl AuditLogDetail {
    /// Compute the detail view for a specific event by replaying the event log.
    /// Returns `None` if the event id is not found.
    pub fn compute(
        events: &[StoreEvent<AppEvent>],
        target_event_id: usize,
        locale: Locale,
    ) -> Option<Self> {
        let target_index = events.iter().position(|e| e.event_id == target_event_id)?;
        let target_event = &events[target_index];

        // ElectionConfig here only seeds the projection's unused `election`
        // field; it does not influence diffing.
        let state_before = replay(&events[..target_index]);
        let state_after = replay(&events[..=target_index]);

        let (old_json, new_json) = extract_old_new(&target_event.payload, &state_before);
        let old_flat = old_json
            .as_ref()
            .map(|v| flatten(v, ""))
            .unwrap_or_default();
        let new_flat = new_json
            .as_ref()
            .map(|v| flatten(v, ""))
            .unwrap_or_default();

        let changes = diff(&old_flat, &new_flat, &state_before, &state_after, locale);
        let entry = AuditLogEntry::new(target_event.clone(), locale);

        Some(AuditLogDetail {
            event_id: entry.event_id,
            description: entry.description,
            details: entry.details,
            subject_id_full: entry.subject_id_full,
            subject_path: entry.subject_path,
            created_at: entry.created_at,
            changes,
        })
    }
}

fn replay(events: &[StoreEvent<AppEvent>]) -> AppStoreData {
    let mut state = AppStoreData::new(ElectionConfig::EK27);
    for event in events {
        state.apply(event.clone());
    }
    state
}

/// Compare two flattened maps and return only the fields that differ,
/// enriched with entity references where keys refer to other entities.
fn diff(
    old: &BTreeMap<String, String>,
    new: &BTreeMap<String, String>,
    state_before: &AppStoreData,
    state_after: &AppStoreData,
    locale: Locale,
) -> Vec<FieldChange> {
    let mut changes = Vec::new();
    let mut all_keys: BTreeMap<&str, ()> = BTreeMap::new();
    for key in old.keys().chain(new.keys()) {
        all_keys.insert(key.as_str(), ());
    }

    for key in all_keys.keys() {
        if EXCLUDED_FIELDS
            .iter()
            .any(|ex| *key == *ex || key.ends_with(&format!(".{ex}")))
        {
            continue;
        }

        let old_val = old.get(*key).cloned().unwrap_or_default();
        let new_val = new.get(*key).cloned().unwrap_or_default();
        if old_val == new_val {
            continue;
        }

        let old_refs = build_refs_for_key(key, &old_val, state_before);
        let new_refs = build_refs_for_key(key, &new_val, state_after);
        changes.push(FieldChange {
            field: translate_field_name(key, locale),
            old_value: old_val,
            new_value: new_val,
            old_refs,
            new_refs,
        });
    }

    changes
}

/// Translate a flattened field name to a human-readable label.
///
/// Uses the leaf segment of the dot-notation path (e.g. `name.first_name` →
/// `first_name`) to look up a translation. Array indices become a 1-indexed
/// suffix on the parent field name (e.g. `candidates.3` → `Candidates #4`).
fn translate_field_name(field: &str, locale: Locale) -> String {
    let leaf = field.rsplit('.').next().unwrap_or(field);
    if let Ok(index) = leaf.parse::<usize>()
        && let Some(parent) = field.rsplit_once('.').map(|(p, _)| p)
    {
        return format!("{} #{}", translate_field_name(parent, locale), index + 1);
    }

    match leaf {
        // Name fields
        "first_name" => trans!("audit_log.detail.fields.first_name", locale),
        "last_name" => trans!("audit_log.detail.fields.last_name", locale),
        "last_name_prefix" => trans!("audit_log.detail.fields.last_name_prefix", locale),
        "initials" => trans!("audit_log.detail.fields.initials", locale),
        // Personal data fields
        "gender" => trans!("audit_log.detail.fields.gender", locale),
        "bsn" | "Bsn" => trans!("audit_log.detail.fields.bsn", locale),
        "date_of_birth" => trans!("audit_log.detail.fields.date_of_birth", locale),
        "place_of_residence" => trans!("audit_log.detail.fields.place_of_residence", locale),
        // Address fields
        "street_name" => trans!("audit_log.detail.fields.street_name", locale),
        "house_number" => trans!("audit_log.detail.fields.house_number", locale),
        "house_number_addition" => {
            trans!("audit_log.detail.fields.house_number_addition", locale)
        }
        "locality" => trans!("audit_log.detail.fields.locality", locale),
        "postal_code" => trans!("audit_log.detail.fields.postal_code", locale),
        "state_or_province" => trans!("audit_log.detail.fields.state_or_province", locale),
        "country" => trans!("audit_log.detail.fields.country", locale),
        // Political group fields
        "long_list_allowed" => trans!("audit_log.detail.fields.long_list_allowed", locale),
        "legal_name" => trans!("audit_log.detail.fields.legal_name", locale),
        "display_name" => trans!("audit_log.detail.fields.display_name", locale),
        // Candidate list fields
        "electoral_districts" => trans!("audit_log.detail.fields.electoral_districts", locale),
        "candidates" => trans!("audit_log.detail.fields.candidates", locale),
        "list_submitter_id" => trans!("audit_log.detail.fields.list_submitter_id", locale),
        "substitute_list_submitter_ids" => {
            trans!(
                "audit_log.detail.fields.substitute_list_submitter_ids",
                locale
            )
        }
        // System event fields
        "person_id" => trans!("audit_log.detail.fields.person_id", locale),
        "political_group_id" => trans!("audit_log.detail.fields.political_group_id", locale),
        "file_name" => trans!("audit_log.detail.fields.file_name", locale),
        "file_size" => trans!("audit_log.detail.fields.file_size", locale),
        "download_path" => trans!("audit_log.detail.fields.download_path", locale),
        "list_id" => trans!("audit_log.detail.fields.list_id", locale),
        // Address type discriminator
        "Dutch" | "International" => trans!("audit_log.detail.fields.address_type", locale),
        // Bsn variant
        "NoneConfirmed" => trans!("audit_log.detail.fields.bsn", locale),
        // Fallback: use the raw field name
        _ => field.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ElectoralDistrict, Locale,
        candidate_lists::CandidateListId,
        common::FullName,
        list_submitters::ListSubmitterId,
        persons::PersonId,
        test_utils::{
            sample_candidate_list, sample_list_submitter, sample_person, sample_political_group,
        },
    };

    const EN: Locale = Locale::En;

    fn empty_state() -> AppStoreData {
        AppStoreData::new(ElectionConfig::EK27)
    }

    // --- diff() ---

    #[test]
    fn diff_detects_changes() {
        let mut old = BTreeMap::new();
        old.insert("name".to_string(), "Alice".to_string());
        old.insert("age".to_string(), "30".to_string());

        let mut new = BTreeMap::new();
        new.insert("name".to_string(), "Bob".to_string());
        new.insert("age".to_string(), "30".to_string());

        let state = empty_state();
        let changes = diff(&old, &new, &state, &state, EN);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "name");
        assert_eq!(changes[0].old_value, "Alice");
        assert_eq!(changes[0].new_value, "Bob");
    }

    #[test]
    fn diff_detects_additions() {
        let old = BTreeMap::new();
        let mut new = BTreeMap::new();
        new.insert("name".to_string(), "Alice".to_string());

        let state = empty_state();
        let changes = diff(&old, &new, &state, &state, EN);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_value, "");
        assert_eq!(changes[0].new_value, "Alice");
    }

    #[test]
    fn diff_detects_removals() {
        let mut old = BTreeMap::new();
        old.insert("name".to_string(), "Alice".to_string());
        let new = BTreeMap::new();

        let state = empty_state();
        let changes = diff(&old, &new, &state, &state, EN);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_value, "Alice");
        assert_eq!(changes[0].new_value, "");
    }

    #[test]
    fn diff_excludes_id_fields() {
        let mut old = BTreeMap::new();
        old.insert("id".to_string(), "abc".to_string());
        old.insert("name".to_string(), "Alice".to_string());

        let mut new = BTreeMap::new();
        new.insert("id".to_string(), "abc".to_string());
        new.insert("name".to_string(), "Bob".to_string());

        let state = empty_state();
        let changes = diff(&old, &new, &state, &state, EN);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "name");
    }

    #[test]
    fn diff_no_changes() {
        let mut old = BTreeMap::new();
        old.insert("name".to_string(), "Alice".to_string());
        let new = old.clone();

        let state = empty_state();
        let changes = diff(&old, &new, &state, &state, EN);
        assert!(changes.is_empty());
    }

    #[test]
    fn diff_excludes_nested_id_fields() {
        let mut old = BTreeMap::new();
        old.insert("person.id".to_string(), "abc".to_string());
        old.insert("person.updated_at".to_string(), "old-ts".to_string());
        old.insert("person.created_at".to_string(), "old-ts".to_string());
        old.insert("person.name".to_string(), "Alice".to_string());

        let mut new = BTreeMap::new();
        new.insert("person.id".to_string(), "abc".to_string());
        new.insert("person.updated_at".to_string(), "new-ts".to_string());
        new.insert("person.created_at".to_string(), "new-ts".to_string());
        new.insert("person.name".to_string(), "Bob".to_string());

        let state = empty_state();
        let changes = diff(&old, &new, &state, &state, EN);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "person.name");
    }

    #[test]
    fn diff_both_empty() {
        let old = BTreeMap::new();
        let new = BTreeMap::new();
        let state = empty_state();
        let changes = diff(&old, &new, &state, &state, EN);
        assert!(changes.is_empty());
    }

    // --- translate_field_name() ---

    #[test]
    fn translate_known_fields() {
        assert_eq!(translate_field_name("first_name", EN), "First name");
        assert_eq!(translate_field_name("last_name", EN), "Last name");
        assert_eq!(translate_field_name("gender", EN), "Gender");
        assert_eq!(translate_field_name("postal_code", EN), "Postal code");
    }

    #[test]
    fn translate_nested_field_uses_leaf() {
        assert_eq!(translate_field_name("name.first_name", EN), "First name");
        assert_eq!(translate_field_name("personal_data.gender", EN), "Gender");
    }

    #[test]
    fn translate_array_index_appends_1_indexed_position() {
        assert_eq!(
            translate_field_name("electoral_districts.0", EN),
            "Electoral districts #1"
        );
        assert_eq!(translate_field_name("candidates.3", EN), "Candidates #4");
    }

    #[test]
    fn translate_unknown_field_returns_raw() {
        assert_eq!(
            translate_field_name("some_unknown_field", EN),
            "some_unknown_field"
        );
    }

    #[test]
    fn translate_dutch_locale() {
        assert_eq!(translate_field_name("first_name", Locale::Nl), "Roepnaam");
    }

    // --- compute() ---

    #[test]
    fn compute_create_event() {
        let person = sample_person(PersonId::new());
        let events = vec![StoreEvent::new(1, AppEvent::CreatePerson(person))];

        let detail = AuditLogDetail::compute(&events, 1, Locale::En).unwrap();
        assert_eq!(detail.event_id, 1);
        assert_eq!(detail.description, "Created person");
        assert!(!detail.changes.is_empty());
        for change in &detail.changes {
            assert!(change.old_value.is_empty());
        }
    }

    #[test]
    fn compute_update_event_shows_diff() {
        let person_id = PersonId::new();
        let person = sample_person(person_id);
        let mut updated_person = person.clone();
        updated_person.name = FullName {
            first_name: Some("Updated".parse().unwrap()),
            ..person.name.clone()
        };

        let events = vec![
            StoreEvent::new(1, AppEvent::CreatePerson(person)),
            StoreEvent::new(2, AppEvent::UpdatePerson(updated_person)),
        ];

        let detail = AuditLogDetail::compute(&events, 2, Locale::En).unwrap();
        let first_name_change = detail
            .changes
            .iter()
            .find(|c| c.field == "First name")
            .expect("should have first_name change");
        assert_eq!(first_name_change.new_value, "Updated");
    }

    #[test]
    fn compute_delete_event() {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        let events = vec![
            StoreEvent::new(1, AppEvent::CreatePerson(person)),
            StoreEvent::new(2, AppEvent::DeletePerson { person_id }),
        ];

        let detail = AuditLogDetail::compute(&events, 2, Locale::En).unwrap();
        assert!(!detail.changes.is_empty());
        for change in &detail.changes {
            assert!(change.new_value.is_empty());
        }
    }

    #[test]
    fn compute_returns_none_for_unknown_event() {
        let events = vec![StoreEvent::new(
            1,
            AppEvent::UpdatePoliticalGroup(sample_political_group()),
        )];
        assert!(AuditLogDetail::compute(&events, 999, Locale::En).is_none());
    }

    #[test]
    fn compute_returns_none_for_empty_events() {
        let events: Vec<StoreEvent<AppEvent>> = vec![];
        assert!(AuditLogDetail::compute(&events, 1, Locale::En).is_none());
    }

    #[test]
    fn compute_system_event_has_no_old_state() {
        let list_id = CandidateListId::new();
        let events = vec![StoreEvent::new(
            1,
            AppEvent::ExportCsv {
                file_name: "export.csv".to_string(),
                file_size: 100,
                list_id,
            },
        )];

        let detail = AuditLogDetail::compute(&events, 1, Locale::En).unwrap();
        for change in &detail.changes {
            assert!(change.old_value.is_empty());
        }
    }

    #[test]
    fn compute_array_of_scalars_diff_is_single_csv_row() {
        let list_id = CandidateListId::new();
        let mut list = sample_candidate_list(list_id);
        list.electoral_districts = vec![ElectoralDistrict::GR];

        let events = vec![
            StoreEvent::new(1, AppEvent::CreateCandidateList(list)),
            StoreEvent::new(
                2,
                AppEvent::UpdateCandidateListDistricts {
                    list_id,
                    electoral_districts: vec![ElectoralDistrict::GR, ElectoralDistrict::FR],
                },
            ),
        ];

        let detail = AuditLogDetail::compute(&events, 2, Locale::En).unwrap();
        let districts: Vec<_> = detail
            .changes
            .iter()
            .filter(|c| c.field == "Electoral districts")
            .collect();

        assert_eq!(districts.len(), 1);
        assert_eq!(districts[0].old_value, "GR");
        assert_eq!(districts[0].new_value, "GR, FR");
    }

    #[test]
    fn compute_id_field_includes_entity_refs() {
        let list_id = CandidateListId::new();
        let old_submitter_id = ListSubmitterId::new();
        let new_submitter_id = ListSubmitterId::new();
        let old_submitter = sample_list_submitter(old_submitter_id);
        let old_submitter_name = old_submitter.name.display();
        let new_submitter = sample_list_submitter(new_submitter_id);
        let new_submitter_name = new_submitter.name.display();

        let mut list = sample_candidate_list(list_id);
        list.list_submitter_id = Some(old_submitter_id);

        let events = vec![
            StoreEvent::new(1, AppEvent::CreateListSubmitter(old_submitter)),
            StoreEvent::new(2, AppEvent::CreateListSubmitter(new_submitter)),
            StoreEvent::new(3, AppEvent::CreateCandidateList(list)),
            StoreEvent::new(
                4,
                AppEvent::UpdateCandidateListSubmitters {
                    list_id,
                    list_submitter_id: Some(new_submitter_id),
                    substitute_list_submitter_ids: vec![],
                },
            ),
        ];

        let detail = AuditLogDetail::compute(&events, 4, Locale::En).unwrap();
        let change = detail
            .changes
            .iter()
            .find(|c| c.field == "List submitter")
            .expect("list_submitter_id change");

        assert_eq!(change.old_refs.len(), 1);
        assert_eq!(change.old_refs[0].id_full, old_submitter_id.to_string());
        assert_eq!(change.old_refs[0].description, old_submitter_name);

        assert_eq!(change.new_refs.len(), 1);
        assert_eq!(change.new_refs[0].id_full, new_submitter_id.to_string());
        assert_eq!(change.new_refs[0].description, new_submitter_name);
    }

    #[test]
    fn compute_candidates_array_renders_changed_positions() {
        let list_id = CandidateListId::new();
        let p1 = sample_person(PersonId::new());
        let p2 = sample_person(PersonId::new());
        let p3 = sample_person(PersonId::new());
        let (p1_id, p2_id, p3_id) = (p1.id, p2.id, p3.id);
        let p1_name = p1.name.display();
        let p2_name = p2.name.display();

        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![p1_id, p2_id, p3_id];

        let events = vec![
            StoreEvent::new(1, AppEvent::CreatePerson(p1)),
            StoreEvent::new(2, AppEvent::CreatePerson(p2)),
            StoreEvent::new(3, AppEvent::CreatePerson(p3)),
            StoreEvent::new(4, AppEvent::CreateCandidateList(list)),
            // Swap positions 0 and 1; position 2 unchanged.
            StoreEvent::new(
                5,
                AppEvent::UpdateCandidateListOrder {
                    list_id,
                    candidates: vec![p2_id, p1_id, p3_id],
                },
            ),
        ];

        let detail = AuditLogDetail::compute(&events, 5, Locale::En).unwrap();
        let candidate_changes: Vec<_> = detail
            .changes
            .iter()
            .filter(|c| c.field.starts_with("Candidates #"))
            .collect();

        assert_eq!(candidate_changes.len(), 2);

        let pos_1 = candidate_changes
            .iter()
            .find(|c| c.field == "Candidates #1")
            .expect("position #1 change");
        assert_eq!(pos_1.old_refs[0].description, p1_name);
        assert_eq!(pos_1.new_refs[0].description, p2_name);

        let pos_2 = candidate_changes
            .iter()
            .find(|c| c.field == "Candidates #2")
            .expect("position #2 change");
        assert_eq!(pos_2.old_refs[0].description, p2_name);
        assert_eq!(pos_2.new_refs[0].description, p1_name);

        assert!(
            !detail.changes.iter().any(|c| c.field == "Candidates #3"),
            "unchanged position should not appear"
        );
    }

    #[test]
    fn compute_non_id_field_has_no_refs() {
        let person_id = PersonId::new();
        let person = sample_person(person_id);
        let mut updated = person.clone();
        updated.name = FullName {
            first_name: Some("Updated".parse().unwrap()),
            ..person.name.clone()
        };

        let events = vec![
            StoreEvent::new(1, AppEvent::CreatePerson(person)),
            StoreEvent::new(2, AppEvent::UpdatePerson(updated)),
        ];

        let detail = AuditLogDetail::compute(&events, 2, Locale::En).unwrap();
        let change = detail
            .changes
            .iter()
            .find(|c| c.field == "First name")
            .expect("first name change");
        assert!(change.old_refs.is_empty());
        assert!(change.new_refs.is_empty());
    }

    #[test]
    fn compute_removed_candidate_ref_resolved_from_state_before() {
        let list_id = CandidateListId::new();
        let person = sample_person(PersonId::new());
        let person_id = person.id;
        let person_name = person.name.display();

        let mut list = sample_candidate_list(list_id);
        list.candidates = vec![person_id];

        let events = vec![
            StoreEvent::new(1, AppEvent::CreatePerson(person)),
            StoreEvent::new(2, AppEvent::CreateCandidateList(list)),
            StoreEvent::new(
                3,
                AppEvent::RemoveCandidateFromCandidateList { list_id, person_id },
            ),
        ];

        let detail = AuditLogDetail::compute(&events, 3, Locale::En).unwrap();
        let change = detail
            .changes
            .iter()
            .find(|c| c.field == "Person ID")
            .expect("person_id change");
        assert_eq!(change.old_refs.len(), 1);
        assert_eq!(change.old_refs[0].description, person_name);
        assert!(change.new_refs.is_empty());
    }
}
