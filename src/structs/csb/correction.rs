use serde::{Deserialize, Serialize};

use crate::{
    common::{DateOfBirth, DisplayName, Initials, LastName, PlaceOfResidence},
    persons::{Person, PersonId},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PersonCorrection {
    Initials(Initials),
    LastName(LastName),
    DateOfBirth(DateOfBirth),
    PlaceOfResidence(PlaceOfResidence),
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

    pub fn details(&self, person_id: PersonId) -> String {
        match self {
            PersonCorrection::Initials(initials) => {
                format!("Person ({person_id}) initials: {initials}")
            }
            PersonCorrection::LastName(last_name) => {
                format!("Person ({person_id}) last name: {last_name}")
            }
            PersonCorrection::DateOfBirth(date_of_birth) => {
                format!("Person ({person_id}) date of birth: {date_of_birth}")
            }
            PersonCorrection::PlaceOfResidence(place_of_residence) => {
                format!("Person ({person_id}) place of residence: {place_of_residence}")
            }
        }
    }
}

/// "Ambtshalve" (ex officio) corrections, done by the CSB based on the BRP and other official records
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Correction {
    DisplayName(DisplayName),
    Person(PersonId, PersonCorrection),
}
