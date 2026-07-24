use std::{collections::HashSet, mem};

use serde::{Deserialize, Serialize};

use crate::{
    Locale,
    common::{DateOfBirth, DisplayName, Initials, LastName, PlaceOfResidence},
    persons::{Person, PersonId},
    structs::audit_log::FieldChange,
    trans,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum PersonCorrection {
    Initials(Initials),
    LastName(LastName),
    DateOfBirth(DateOfBirth),
    PlaceOfResidence(PlaceOfResidence),
}

/// Representing a set of corrections on a single person. 
/// - [`None`] means no correction.
/// - [`Some(_)`] contains the corrected value
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PersonCorrectionDelta {
    // NOTE: deliberately not public, mutate, and create through impl methods
    changes: HashSet<PersonCorrection>
}

impl PersonCorrectionDelta {
    pub fn new() ->  PersonCorrectionDelta {
        PersonCorrectionDelta { changes: HashSet::new() }
    }

    /// Add a correction to this delta
    /// Replaces the previous [`PersonCorrection`] variant in the delta if present
    pub fn add_correction(&mut self, correction: PersonCorrection) {
        self.changes.retain(|c| mem::discriminant(c) != mem::discriminant(&correction));
        self.changes.insert(correction);
    }
}

impl PersonCorrection {
    pub fn apply(self, person: &mut Person) {
        match self {
            PersonCorrection::Initials(initials) => {
                person.name.initials = initials;
            }
            PersonCorrection::LastName(last_name) => {
                person.name.last_name = last_name;
            }
            PersonCorrection::DateOfBirth(date_of_birth) => {
                person.personal_data.date_of_birth = Some(date_of_birth);
            }
            PersonCorrection::PlaceOfResidence(place_of_residence) => {
                person.personal_data.place_of_residence = Some(place_of_residence);
            }
        }
    }

    pub fn change(&self, locale: Locale) -> FieldChange {
        let (field, new_value) = match self {
            PersonCorrection::Initials(v) => (
                trans!("audit_log.detail.fields.initials", locale),
                v.to_string(),
            ),
            PersonCorrection::LastName(v) => (
                trans!("audit_log.detail.fields.last_name", locale),
                v.to_string(),
            ),
            PersonCorrection::DateOfBirth(v) => (
                trans!("audit_log.detail.fields.date_of_birth", locale),
                v.to_string(),
            ),
            PersonCorrection::PlaceOfResidence(v) => (
                trans!("audit_log.detail.fields.place_of_residence", locale),
                v.to_string(),
            ),
        };
        FieldChange::Regular {
            field,
            old_value: String::new(),
            new_value,
        }
    }
}

impl Correction {
    pub fn change(&self, locale: Locale) -> FieldChange {
        match self {
            Correction::DisplayName(v) => FieldChange::Regular {
                field: trans!("audit_log.detail.fields.display_name", locale),
                old_value: String::new(),
                new_value: v.to_string(),
            },
            Correction::Person(_, person_correction) => person_correction.change(locale),
        }
    }
}

/// "Ambtshalve" (ex officio) corrections, done by the CSB based on the BRP and other official records
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Correction {
    DisplayName(DisplayName),
    Person(PersonId, PersonCorrection),
}
