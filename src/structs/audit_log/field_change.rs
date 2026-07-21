/// A single field-level change in an audit log event.
///
/// When the field is an entity ID (or list of IDs), `old_refs` / `new_refs`
/// carry resolved references so the template can render abbreviated clickable
/// links plus a human-readable description. Otherwise the raw `old_value` /
/// `new_value` strings are rendered.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum FieldChange {
    Regular {
        field: String,
        old_value: String,
        new_value: String,
    },
    Entities {
        field: String,
        old_refs: Vec<EntityRef>,
        new_refs: Vec<EntityRef>,
    },
}

impl FieldChange {
    pub fn change_kind(&self) -> &'static str {
        match self {
            FieldChange::Regular { old_value, .. } if old_value.is_empty() => "added",
            FieldChange::Regular { new_value, .. } if new_value.is_empty() => "removed",
            FieldChange::Regular { .. } => "changed",
            FieldChange::Entities { old_refs, .. } if old_refs.is_empty() => "added",
            FieldChange::Entities { new_refs, .. } if new_refs.is_empty() => "removed",
            FieldChange::Entities { .. } => "changed",
        }
    }
}

/// A reference to another entity mentioned inside a diff value. Rendered in
/// the template as an abbreviated link + the entity's description.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct EntityRef {
    pub id_full: String,
    pub description: String,
}
