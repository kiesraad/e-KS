use serde::{Deserialize, Serialize};
use strum::AsRefStr;

use crate::persons::Person;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, AsRefStr, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PersonSort {
    #[default]
    LastName,
    FirstName,
    Initials,
    Gender,
    PlaceOfResidence,
    CreatedAt,
    UpdatedAt,
}

pub fn compare_persons(a: &Person, b: &Person, sort_field: &PersonSort) -> std::cmp::Ordering {
    match sort_field {
        PersonSort::LastName => a
            .last_name
            .cmp(&b.last_name)
            .then_with(|| cmp_option_string(&a.last_name_prefix, &b.last_name_prefix))
            .then_with(|| a.initials.cmp(&b.initials))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::FirstName => cmp_option_string(&a.first_name, &b.first_name)
            .then_with(|| a.last_name.cmp(&b.last_name))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::Initials => a
            .initials
            .cmp(&b.initials)
            .then_with(|| a.last_name.cmp(&b.last_name))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::Gender => a
            .gender
            .cmp(&b.gender)
            .then_with(|| a.last_name.cmp(&b.last_name))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::PlaceOfResidence => {
            cmp_option_string(&a.place_of_residence, &b.place_of_residence)
                .then_with(|| a.last_name.cmp(&b.last_name))
                .then_with(|| a.id.cmp(&b.id))
        }
        PersonSort::CreatedAt => a
            .created_at
            .cmp(&b.created_at)
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::UpdatedAt => a
            .updated_at
            .cmp(&b.updated_at)
            .then_with(|| a.id.cmp(&b.id)),
    }
}

fn cmp_option_string(a: &Option<String>, b: &Option<String>) -> std::cmp::Ordering {
    a.as_deref()
        .unwrap_or_default()
        .cmp(b.as_deref().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persons::{Gender, PersonId};
    use chrono::{TimeZone, Utc};
    use std::cmp::Ordering;
    use uuid::Uuid;

    fn timestamp(day: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0)
            .single()
            .unwrap()
    }

    fn base_person(id: u128) -> Person {
        Person {
            id: PersonId::from(Uuid::from_u128(id)),
            last_name: "Smith".to_string(),
            initials: "A.B.".to_string(),
            created_at: timestamp(1),
            updated_at: timestamp(1),
            ..Default::default()
        }
    }

    #[test]
    fn cmp_option_string_treats_none_as_empty() {
        let none_value = None;
        let empty = Some(String::new());
        let value = Some("A".to_string());

        assert_eq!(cmp_option_string(&none_value, &empty), Ordering::Equal);
        assert_eq!(cmp_option_string(&none_value, &value), Ordering::Less);
        assert_eq!(cmp_option_string(&value, &none_value), Ordering::Greater);
    }

    #[test]
    fn compare_last_name_uses_prefix_initials_and_id_tiebreakers() {
        let mut a = base_person(1);
        let mut b = base_person(2);

        a.last_name = "Adams".to_string();
        b.last_name = "Brown".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::LastName),
            Ordering::Less
        );

        b.last_name = "Adams".to_string();
        a.last_name_prefix = None;
        b.last_name_prefix = Some("van".to_string());
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::LastName),
            Ordering::Less
        );

        b.last_name_prefix = None;
        a.initials = "A.A.".to_string();
        b.initials = "B.B.".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::LastName),
            Ordering::Less
        );

        b.initials = "A.A.".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::LastName),
            Ordering::Less
        );
    }

    #[test]
    fn compare_first_name_then_last_name_and_id() {
        let mut a = base_person(1);
        let mut b = base_person(2);

        a.first_name = None;
        b.first_name = Some("Adam".to_string());
        a.last_name = "Zulu".to_string();
        b.last_name = "Alpha".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::FirstName),
            Ordering::Less
        );

        a.first_name = Some("Bob".to_string());
        b.first_name = Some("Bob".to_string());
        a.last_name = "Alpha".to_string();
        b.last_name = "Zulu".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::FirstName),
            Ordering::Less
        );

        b.last_name = "Alpha".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::FirstName),
            Ordering::Less
        );
    }

    #[test]
    fn compare_initials_then_last_name_and_id() {
        let mut a = base_person(1);
        let mut b = base_person(2);

        a.initials = "A.A.".to_string();
        b.initials = "B.B.".to_string();
        a.last_name = "Zulu".to_string();
        b.last_name = "Alpha".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::Initials),
            Ordering::Less
        );

        b.initials = "A.A.".to_string();
        a.last_name = "Alpha".to_string();
        b.last_name = "Zulu".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::Initials),
            Ordering::Less
        );

        b.last_name = "Alpha".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::Initials),
            Ordering::Less
        );
    }

    #[test]
    fn compare_gender_then_last_name_and_id() {
        let mut a = base_person(1);
        let mut b = base_person(2);

        a.gender = None;
        b.gender = Some(Gender::Female);
        a.last_name = "Zulu".to_string();
        b.last_name = "Alpha".to_string();
        assert_eq!(compare_persons(&a, &b, &PersonSort::Gender), Ordering::Less);

        a.gender = Some(Gender::Female);
        b.gender = Some(Gender::Male);
        a.last_name = "Zulu".to_string();
        b.last_name = "Alpha".to_string();
        assert_eq!(compare_persons(&a, &b, &PersonSort::Gender), Ordering::Less);

        a.gender = Some(Gender::Female);
        b.gender = Some(Gender::Female);
        a.last_name = "Alpha".to_string();
        b.last_name = "Zulu".to_string();
        assert_eq!(compare_persons(&a, &b, &PersonSort::Gender), Ordering::Less);
    }

    #[test]
    fn compare_place_of_residence_then_last_name_and_id() {
        let mut a = base_person(1);
        let mut b = base_person(2);

        a.place_of_residence = None;
        b.place_of_residence = Some("Utrecht".to_string());
        a.last_name = "Zulu".to_string();
        b.last_name = "Alpha".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::PlaceOfResidence),
            Ordering::Less
        );

        a.place_of_residence = Some("Amsterdam".to_string());
        b.place_of_residence = Some("Amsterdam".to_string());
        a.last_name = "Alpha".to_string();
        b.last_name = "Zulu".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::PlaceOfResidence),
            Ordering::Less
        );

        b.last_name = "Alpha".to_string();
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::PlaceOfResidence),
            Ordering::Less
        );
    }

    #[test]
    fn compare_created_at_and_updated_at_use_id_tiebreaker() {
        let mut a = base_person(1);
        let mut b = base_person(2);

        a.created_at = timestamp(1);
        b.created_at = timestamp(2);
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::CreatedAt),
            Ordering::Less
        );

        b.created_at = timestamp(1);
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::CreatedAt),
            Ordering::Less
        );

        a.updated_at = timestamp(3);
        b.updated_at = timestamp(4);
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::UpdatedAt),
            Ordering::Less
        );

        b.updated_at = timestamp(3);
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::UpdatedAt),
            Ordering::Less
        );
    }
}
