use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::{
    AppEvent, AppStoreData, Locale,
    store::{StoreData, StoreEvent},
    trans,
};

use super::audit_log_entry::AuditLogEntry;

/// A single field-level change in an audit log event.
pub struct FieldChange {
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}

/// Detailed view of an audit log event, including field-level changes.
pub struct AuditLogDetail {
    pub event_id: usize,
    pub description: String,
    pub details: String,
    pub subject_id: String,
    pub subject_id_full: String,
    pub subject_path: String,
    pub created_at: DateTime<Utc>,
    pub changes: Vec<FieldChange>,
}

/// Fields to exclude from the diff (metadata, not meaningful changes).
const EXCLUDED_FIELDS: &[&str] = &["id", "updated_at", "created_at"];

impl AuditLogDetail {
    /// Compute the detail view for a specific event by replaying the event log.
    ///
    /// Returns `None` if the event_id is not found.
    pub fn compute(
        events: &[StoreEvent<AppEvent>],
        target_event_id: usize,
        locale: Locale,
    ) -> Option<Self> {
        let target_index = events.iter().position(|e| e.event_id == target_event_id)?;

        let target_event = &events[target_index];

        // Replay all events before the target to reconstruct the prior state.
        let mut state_before = AppStoreData::default();
        for event in &events[..target_index] {
            state_before.apply(event.clone());
        }

        let (old_json, new_json) = extract_old_new(&target_event.payload, &state_before);

        let old_flat = old_json
            .as_ref()
            .map(|v| flatten(v, ""))
            .unwrap_or_default();
        let new_flat = new_json
            .as_ref()
            .map(|v| flatten(v, ""))
            .unwrap_or_default();

        let changes = diff(&old_flat, &new_flat, locale);

        let entry = AuditLogEntry::new(target_event.clone(), locale);

        Some(AuditLogDetail {
            event_id: entry.event_id,
            description: entry.description,
            details: entry.details,
            subject_id: entry.subject_id,
            subject_id_full: entry.subject_id_full,
            subject_path: entry.subject_path,
            created_at: entry.created_at,
            changes,
        })
    }
}

/// Recursively flatten a JSON value into dot-notation key-value pairs.
fn flatten(value: &serde_json::Value, prefix: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();

    match value {
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                map.extend(flatten(val, &full_key));
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let full_key = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{prefix}.{i}")
                };
                map.extend(flatten(val, &full_key));
            }
        }
        serde_json::Value::Null => {
            if !prefix.is_empty() {
                map.insert(prefix.to_string(), String::new());
            }
        }
        serde_json::Value::Bool(b) => {
            if !prefix.is_empty() {
                map.insert(prefix.to_string(), b.to_string());
            }
        }
        serde_json::Value::Number(n) => {
            if !prefix.is_empty() {
                map.insert(prefix.to_string(), n.to_string());
            }
        }
        serde_json::Value::String(s) => {
            if !prefix.is_empty() {
                map.insert(prefix.to_string(), s.clone());
            }
        }
    }

    map
}

/// Compare two flattened maps and return only the fields that differ.
fn diff(
    old: &BTreeMap<String, String>,
    new: &BTreeMap<String, String>,
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

        if old_val != new_val {
            changes.push(FieldChange {
                field: translate_field_name(key, locale),
                old_value: old_val,
                new_value: new_val,
            });
        }
    }

    changes
}

/// Translate a flattened field name to a human-readable label.
///
/// Uses the leaf segment of the dot-notation path (e.g. `name.first_name` → `first_name`)
/// to look up a translation. Array indices are stripped to show the parent field name
/// (e.g. `electoral_districts.0` → `electoral_districts`).
fn translate_field_name(field: &str, locale: Locale) -> String {
    // For array entries like "electoral_districts.0", use the parent name
    let leaf = field.rsplit('.').next().unwrap_or(field);
    let lookup = if leaf.chars().all(|c| c.is_ascii_digit()) {
        // Array index — use the parent segment
        field.rsplit('.').nth(1).unwrap_or(field)
    } else {
        leaf
    };

    match lookup {
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

/// Extract old and new JSON representations for a given event.
///
/// For create events, old is None and new contains the created entity.
/// For update events, old is the entity from state_before and new is the updated entity.
/// For delete events, old is the entity from state_before and new is None.
/// For system events, old is None and new contains the event fields.
fn extract_old_new(
    event: &AppEvent,
    state_before: &AppStoreData,
) -> (Option<serde_json::Value>, Option<serde_json::Value>) {
    match event {
        // Create events: no old state
        AppEvent::CreatePerson(person) => (None, serde_json::to_value(person).ok()),
        AppEvent::CreatePersonPersonalData {
            person_id: _,
            name,
            personal_data,
        } => {
            let new_val = serde_json::json!({
                "name": serde_json::to_value(name).unwrap_or_default(),
                "personal_data": serde_json::to_value(personal_data).unwrap_or_default(),
            });
            (None, Some(new_val))
        }
        AppEvent::CreateCandidateList(cl) => (None, serde_json::to_value(cl).ok()),
        AppEvent::CreateAuthorisedAgent(aa) => (None, serde_json::to_value(aa).ok()),
        AppEvent::CreateListSubmitter(ls) => (None, serde_json::to_value(ls).ok()),
        AppEvent::CreateSubstituteSubmitter(ss) => (None, serde_json::to_value(ss).ok()),

        // Update events (full entity): old from state, new from payload
        AppEvent::UpdatePerson(person) => {
            let old = state_before
                .persons
                .get(&person.id)
                .and_then(|p| serde_json::to_value(p).ok());
            (old, serde_json::to_value(person).ok())
        }
        AppEvent::UpdateAuthorisedAgent(aa) => {
            let old = state_before
                .authorised_agents
                .get(&aa.id)
                .and_then(|a| serde_json::to_value(a).ok());
            (old, serde_json::to_value(aa).ok())
        }
        AppEvent::UpdateListSubmitter(ls) => {
            let old = state_before
                .list_submitters
                .get(&ls.id)
                .and_then(|l| serde_json::to_value(l).ok());
            (old, serde_json::to_value(ls).ok())
        }
        AppEvent::UpdateSubstituteSubmitter(ss) => {
            let old = state_before
                .substitute_submitters
                .get(&ss.id)
                .and_then(|s| serde_json::to_value(s).ok());
            (old, serde_json::to_value(ss).ok())
        }
        AppEvent::UpdatePoliticalGroup(pg) => {
            let old = serde_json::to_value(&state_before.political_group).ok();
            (old, serde_json::to_value(pg).ok())
        }

        // Update events (partial): old from relevant sub-entity, new from payload fields
        AppEvent::UpdatePersonPersonalData {
            person_id,
            name,
            personal_data,
        } => {
            let old = state_before.persons.get(person_id).map(|p| {
                serde_json::json!({
                    "name": serde_json::to_value(&p.name).unwrap_or_default(),
                    "personal_data": serde_json::to_value(&p.personal_data).unwrap_or_default(),
                })
            });
            let new_val = serde_json::json!({
                "name": serde_json::to_value(name).unwrap_or_default(),
                "personal_data": serde_json::to_value(personal_data).unwrap_or_default(),
            });
            (old, Some(new_val))
        }
        AppEvent::UpdatePersonAddress { person_id, address } => {
            let old = state_before
                .persons
                .get(person_id)
                .and_then(|p| serde_json::to_value(&p.address).ok());
            (old, serde_json::to_value(address).ok())
        }
        AppEvent::UpdatePersonRepresentative {
            person_id,
            representative,
        } => {
            let old = state_before
                .persons
                .get(person_id)
                .and_then(|p| serde_json::to_value(&p.representative).ok());
            (old, serde_json::to_value(representative).ok())
        }
        AppEvent::UpdateCandidateListDistricts {
            list_id,
            electoral_districts,
        } => {
            let old = state_before
                .candidate_lists
                .get(list_id)
                .and_then(|cl| serde_json::to_value(&cl.electoral_districts).ok());
            (old, serde_json::to_value(electoral_districts).ok())
        }
        AppEvent::UpdateCandidateListOrder {
            list_id,
            candidates,
        } => {
            let old = state_before
                .candidate_lists
                .get(list_id)
                .and_then(|cl| serde_json::to_value(&cl.candidates).ok());
            (old, serde_json::to_value(candidates).ok())
        }
        AppEvent::UpdateCandidateListSubmitters {
            list_id,
            list_submitter_id,
            substitute_list_submitter_ids,
        } => {
            let old = state_before.candidate_lists.get(list_id).map(|cl| {
                serde_json::json!({
                    "list_submitter_id": serde_json::to_value(cl.list_submitter_id).unwrap_or_default(),
                    "substitute_list_submitter_ids": serde_json::to_value(&cl.substitute_list_submitter_ids).unwrap_or_default(),
                })
            });
            let new_val = serde_json::json!({
                "list_submitter_id": serde_json::to_value(list_submitter_id).unwrap_or_default(),
                "substitute_list_submitter_ids": serde_json::to_value(substitute_list_submitter_ids).unwrap_or_default(),
            });
            (old, Some(new_val))
        }

        // Add/remove candidate: show the person_id that was added/removed
        AppEvent::AddCandidateToCandidateList { person_id, .. } => {
            let new_val = serde_json::json!({ "person_id": person_id.to_string() });
            (None, Some(new_val))
        }
        AppEvent::RemoveCandidateFromCandidateList { person_id, .. } => {
            let old_val = serde_json::json!({ "person_id": person_id.to_string() });
            (Some(old_val), None)
        }

        // Delete events: old from state, no new state
        AppEvent::DeletePerson { person_id } => {
            let old = state_before
                .persons
                .get(person_id)
                .and_then(|p| serde_json::to_value(p).ok());
            (old, None)
        }
        AppEvent::DeleteCandidateList(cl_id) => {
            let old = state_before
                .candidate_lists
                .get(cl_id)
                .and_then(|cl| serde_json::to_value(cl).ok());
            (old, None)
        }
        AppEvent::DeleteAuthorisedAgent(aa_id) => {
            let old = state_before
                .authorised_agents
                .get(aa_id)
                .and_then(|a| serde_json::to_value(a).ok());
            (old, None)
        }
        AppEvent::DeleteListSubmitter {
            list_submitter_id: ls_id,
        } => {
            let old = state_before
                .list_submitters
                .get(ls_id)
                .and_then(|l| serde_json::to_value(l).ok());
            (old, None)
        }
        AppEvent::DeleteSubstituteSubmitter {
            substitute_submitter_id: ss_id,
        } => {
            let old = state_before
                .substitute_submitters
                .get(ss_id)
                .and_then(|s| serde_json::to_value(s).ok());
            (old, None)
        }

        // System events: informational only
        AppEvent::DeveloperLogin { political_group_id } => {
            let val = serde_json::json!({ "political_group_id": political_group_id.to_string() });
            (None, Some(val))
        }
        AppEvent::DownloadFile {
            file_name,
            download_path,
            list_id,
        } => {
            let val = serde_json::json!({
                "file_name": file_name,
                "download_path": download_path,
                "list_id": list_id.to_string(),
            });
            (None, Some(val))
        }
        AppEvent::ExportCsv {
            file_name,
            file_size,
            list_id,
        }
        | AppEvent::ImportCsv {
            file_name,
            file_size,
            list_id,
        } => {
            let val = serde_json::json!({
                "file_name": file_name,
                "file_size": file_size,
                "list_id": list_id.to_string(),
            });
            (None, Some(val))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Locale, PoliticalGroupId,
        common::FullName,
        persons::PersonId,
        test_utils::{sample_person, sample_political_group},
    };

    const EN: Locale = Locale::En;

    #[test]
    fn flatten_simple_object() {
        let val = serde_json::json!({
            "name": "Alice",
            "age": 30
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("name").unwrap(), "Alice");
        assert_eq!(flat.get("age").unwrap(), "30");
    }

    #[test]
    fn flatten_nested_object() {
        let val = serde_json::json!({
            "name": {
                "first": "Alice",
                "last": "Smith"
            }
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("name.first").unwrap(), "Alice");
        assert_eq!(flat.get("name.last").unwrap(), "Smith");
    }

    #[test]
    fn flatten_array() {
        let val = serde_json::json!({
            "items": ["a", "b", "c"]
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("items.0").unwrap(), "a");
        assert_eq!(flat.get("items.1").unwrap(), "b");
        assert_eq!(flat.get("items.2").unwrap(), "c");
    }

    #[test]
    fn flatten_null_value() {
        let val = serde_json::json!({
            "field": null
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("field").unwrap(), "");
    }

    #[test]
    fn diff_detects_changes() {
        let mut old = BTreeMap::new();
        old.insert("name".to_string(), "Alice".to_string());
        old.insert("age".to_string(), "30".to_string());

        let mut new = BTreeMap::new();
        new.insert("name".to_string(), "Bob".to_string());
        new.insert("age".to_string(), "30".to_string());

        let changes = diff(&old, &new, EN);
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

        let changes = diff(&old, &new, EN);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "name");
        assert_eq!(changes[0].old_value, "");
        assert_eq!(changes[0].new_value, "Alice");
    }

    #[test]
    fn diff_detects_removals() {
        let mut old = BTreeMap::new();
        old.insert("name".to_string(), "Alice".to_string());
        let new = BTreeMap::new();

        let changes = diff(&old, &new, EN);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "name");
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

        let changes = diff(&old, &new, EN);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "name");
    }

    #[test]
    fn compute_create_event() {
        let person = sample_person(PersonId::new());
        let events = vec![StoreEvent::new(1, AppEvent::CreatePerson(person.clone()))];

        let detail = AuditLogDetail::compute(&events, 1, Locale::En).unwrap();
        assert_eq!(detail.event_id, 1);
        assert_eq!(detail.description, "Created person");
        // All fields should appear as additions (old_value empty)
        assert!(!detail.changes.is_empty());
        for change in &detail.changes {
            assert!(
                change.old_value.is_empty(),
                "create event old_value should be empty for field {}",
                change.field
            );
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
        assert_eq!(detail.event_id, 2);

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
        // All fields should appear as removals (new_value empty)
        assert!(!detail.changes.is_empty());
        for change in &detail.changes {
            assert!(
                change.new_value.is_empty(),
                "delete event new_value should be empty for field {}",
                change.field
            );
        }
    }

    #[test]
    fn compute_returns_none_for_unknown_event() {
        let events = vec![StoreEvent::new(
            1,
            AppEvent::UpdatePoliticalGroup(sample_political_group(PoliticalGroupId::new())),
        )];

        assert!(AuditLogDetail::compute(&events, 999, Locale::En).is_none());
    }

    #[test]
    fn flatten_boolean_value() {
        let val = serde_json::json!({
            "active": true,
            "deleted": false
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("active").unwrap(), "true");
        assert_eq!(flat.get("deleted").unwrap(), "false");
    }

    #[test]
    fn flatten_deeply_nested() {
        let val = serde_json::json!({
            "a": { "b": { "c": "deep" } }
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("a.b.c").unwrap(), "deep");
        assert_eq!(flat.len(), 1);
    }

    #[test]
    fn flatten_empty_object() {
        let val = serde_json::json!({});
        let flat = flatten(&val, "");
        assert!(flat.is_empty());
    }

    #[test]
    fn flatten_empty_array() {
        let val = serde_json::json!({ "items": [] });
        let flat = flatten(&val, "");
        assert!(flat.is_empty());
    }

    #[test]
    fn flatten_mixed_types() {
        let val = serde_json::json!({
            "name": "test",
            "count": 42,
            "active": true,
            "tags": ["a"],
            "meta": null
        });
        let flat = flatten(&val, "");
        assert_eq!(flat.get("name").unwrap(), "test");
        assert_eq!(flat.get("count").unwrap(), "42");
        assert_eq!(flat.get("active").unwrap(), "true");
        assert_eq!(flat.get("tags.0").unwrap(), "a");
        assert_eq!(flat.get("meta").unwrap(), "");
    }

    #[test]
    fn flatten_root_scalar_ignored() {
        // Root-level scalars without a prefix are not inserted
        let val = serde_json::json!("hello");
        let flat = flatten(&val, "");
        assert!(flat.is_empty());
    }

    #[test]
    fn flatten_with_prefix() {
        let val = serde_json::json!({ "x": 1 });
        let flat = flatten(&val, "root");
        assert_eq!(flat.get("root.x").unwrap(), "1");
    }

    #[test]
    fn diff_no_changes() {
        let mut old = BTreeMap::new();
        old.insert("name".to_string(), "Alice".to_string());

        let new = old.clone();
        let changes = diff(&old, &new, EN);
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

        let changes = diff(&old, &new, EN);
        // Only `name` change should remain; id, updated_at, created_at are excluded
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "person.name");
    }

    #[test]
    fn diff_both_empty() {
        let old = BTreeMap::new();
        let new = BTreeMap::new();
        let changes = diff(&old, &new, EN);
        assert!(changes.is_empty());
    }

    #[test]
    fn translate_known_fields() {
        assert_eq!(translate_field_name("first_name", EN), "First name");
        assert_eq!(translate_field_name("last_name", EN), "Last name");
        assert_eq!(translate_field_name("gender", EN), "Gender");
        assert_eq!(translate_field_name("postal_code", EN), "Postal code");
    }

    #[test]
    fn translate_nested_field_uses_leaf() {
        // "name.first_name" should resolve via the leaf "first_name"
        assert_eq!(translate_field_name("name.first_name", EN), "First name");
        assert_eq!(translate_field_name("personal_data.gender", EN), "Gender");
    }

    #[test]
    fn translate_array_index_uses_parent() {
        // "electoral_districts.0" → leaf is "0" (all digits) → use parent "electoral_districts"
        assert_eq!(
            translate_field_name("electoral_districts.0", EN),
            "Electoral districts"
        );
        assert_eq!(translate_field_name("candidates.3", EN), "Candidates");
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

    #[test]
    fn compute_system_event_has_no_old_state() {
        let list_id = crate::candidate_lists::CandidateListId::new();
        let events = vec![StoreEvent::new(
            1,
            AppEvent::ExportCsv {
                file_name: "export.csv".to_string(),
                file_size: 100,
                list_id,
            },
        )];

        let detail = AuditLogDetail::compute(&events, 1, Locale::En).unwrap();
        // System events: all changes are additions (no old state)
        for change in &detail.changes {
            assert!(
                change.old_value.is_empty(),
                "system event old_value should be empty for field {}",
                change.field
            );
        }
    }

    #[test]
    fn compute_returns_none_for_empty_events() {
        let events: Vec<StoreEvent<AppEvent>> = vec![];
        assert!(AuditLogDetail::compute(&events, 1, Locale::En).is_none());
    }
}
