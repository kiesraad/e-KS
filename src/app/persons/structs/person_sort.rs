use serde::{Deserialize, Serialize};

use crate::{OptionStringExt, persons::Person};

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonSort {
    #[default]
    LastName,
    FirstName,
    Initials,
    Gender,
    PlaceOfResidence,
    UpdatedAt,
}

pub fn compare_persons(a: &Person, b: &Person, sort_field: &PersonSort) -> std::cmp::Ordering {
    match sort_field {
        PersonSort::LastName => a
            .name
            .last_name
            .cmp(&b.name.last_name)
            .then_with(|| {
                a.name
                    .last_name_prefix
                    .as_str_or_empty()
                    .cmp(b.name.last_name_prefix.as_str_or_empty())
            })
            .then_with(|| a.name.initials.cmp(&b.name.initials))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::FirstName => a
            .first_name
            .as_str_or_empty()
            .cmp(b.first_name.as_str_or_empty())
            .then_with(|| a.name.last_name.cmp(&b.name.last_name))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::Initials => a
            .name
            .initials
            .cmp(&b.name.initials)
            .then_with(|| a.name.last_name.cmp(&b.name.last_name))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::Gender => a
            .gender
            .cmp(&b.gender)
            .then_with(|| a.name.last_name.cmp(&b.name.last_name))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::PlaceOfResidence => a
            .place_of_residence
            .as_str_or_empty()
            .cmp(b.place_of_residence.as_str_or_empty())
            .then_with(|| a.name.last_name.cmp(&b.name.last_name))
            .then_with(|| a.id.cmp(&b.id)),
        PersonSort::UpdatedAt => a
            .updated_at
            .cmp(&b.updated_at)
            .then_with(|| a.id.cmp(&b.id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        common::{
            FirstName, FullName, Gender, Initials, LastName, LastNamePrefix, PlaceOfResidence,
        },
        persons::PersonId,
    };
    use chrono::{TimeZone, Utc};
    use std::cmp::Ordering;
    use uuid::Uuid;

    fn timestamp(day: u32) -> crate::common::UtcDateTime {
        crate::common::UtcDateTime::from(
            Utc.with_ymd_and_hms(2026, 1, day, 0, 0, 0)
                .single()
                .unwrap(),
        )
    }

    fn base_person(id: u128) -> Person {
        Person {
            id: PersonId::from(Uuid::from_u128(id)),
            name: FullName {
                last_name: "Smith".parse::<LastName>().expect("last name"),
                last_name_prefix: None,
                initials: "A.B.".parse::<Initials>().expect("initials"),
            },
            ..Default::default()
        }
    }

    #[test]
    fn compare_last_name_uses_prefix_initials_and_id_tiebreakers() {
        let mut a = base_person(1);
        let mut b = base_person(2);

        a.name.last_name = "Adams".parse::<LastName>().expect("last name");
        b.name.last_name = "Brown".parse::<LastName>().expect("last name");
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::LastName),
            Ordering::Less
        );

        b.name.last_name = "Adams".parse::<LastName>().expect("last name");
        a.name.last_name_prefix = None;
        b.name.last_name_prefix = Some("van".parse::<LastNamePrefix>().expect("last name prefix"));
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::LastName),
            Ordering::Less
        );

        b.name.last_name_prefix = None;
        a.name.initials = "A.A.".parse::<Initials>().expect("initials");
        b.name.initials = "B.B.".parse::<Initials>().expect("initials");
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::LastName),
            Ordering::Less
        );

        b.name.initials = "A.A.".parse::<Initials>().expect("initials");
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
        b.first_name = Some("Adam".parse::<FirstName>().expect("first name"));
        a.name.last_name = "Zulu".parse::<LastName>().expect("last name");
        b.name.last_name = "Alpha".parse::<LastName>().expect("last name");
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::FirstName),
            Ordering::Less
        );

        a.first_name = Some("Bob".parse::<FirstName>().expect("first name"));
        b.first_name = Some("Bob".parse::<FirstName>().expect("first name"));
        a.name.last_name = "Alpha".parse::<LastName>().expect("last name");
        b.name.last_name = "Zulu".parse::<LastName>().expect("last name");
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::FirstName),
            Ordering::Less
        );

        b.name.last_name = "Alpha".parse::<LastName>().expect("last name");
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::FirstName),
            Ordering::Less
        );
    }

    #[test]
    fn compare_initials_then_last_name_and_id() {
        let mut a = base_person(1);
        let mut b = base_person(2);

        a.name.initials = "A.A.".parse::<Initials>().expect("initials");
        b.name.initials = "B.B.".parse::<Initials>().expect("initials");
        a.name.last_name = "Zulu".parse::<LastName>().expect("last name");
        b.name.last_name = "Alpha".parse::<LastName>().expect("last name");
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::Initials),
            Ordering::Less
        );

        b.name.initials = "A.A.".parse::<Initials>().expect("initials");
        a.name.last_name = "Alpha".parse::<LastName>().expect("last name");
        b.name.last_name = "Zulu".parse::<LastName>().expect("last name");
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::Initials),
            Ordering::Less
        );

        b.name.last_name = "Alpha".parse::<LastName>().expect("last name");
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
        a.name.last_name = "Zulu".parse::<LastName>().expect("last name");
        b.name.last_name = "Alpha".parse::<LastName>().expect("last name");
        assert_eq!(compare_persons(&a, &b, &PersonSort::Gender), Ordering::Less);

        a.gender = Some(Gender::Female);
        b.gender = Some(Gender::Male);
        a.name.last_name = "Zulu".parse::<LastName>().expect("last name");
        b.name.last_name = "Alpha".parse::<LastName>().expect("last name");
        assert_eq!(compare_persons(&a, &b, &PersonSort::Gender), Ordering::Less);

        a.gender = Some(Gender::Female);
        b.gender = Some(Gender::Female);
        a.name.last_name = "Alpha".parse::<LastName>().expect("last name");
        b.name.last_name = "Zulu".parse::<LastName>().expect("last name");
        assert_eq!(compare_persons(&a, &b, &PersonSort::Gender), Ordering::Less);
    }

    #[test]
    fn compare_place_of_residence_then_last_name_and_id() {
        let mut a = base_person(1);
        let mut b = base_person(2);

        a.place_of_residence = None;
        b.place_of_residence = Some(
            "Utrecht"
                .parse::<PlaceOfResidence>()
                .expect("place of residence"),
        );
        a.name.last_name = "Zulu".parse::<LastName>().expect("last name");
        b.name.last_name = "Alpha".parse::<LastName>().expect("last name");
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::PlaceOfResidence),
            Ordering::Less
        );

        a.place_of_residence = Some(
            "Amsterdam"
                .parse::<PlaceOfResidence>()
                .expect("place of residence"),
        );
        b.place_of_residence = Some(
            "Amsterdam"
                .parse::<PlaceOfResidence>()
                .expect("place of residence"),
        );
        a.name.last_name = "Alpha".parse::<LastName>().expect("last name");
        b.name.last_name = "Zulu".parse::<LastName>().expect("last name");
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::PlaceOfResidence),
            Ordering::Less
        );

        b.name.last_name = "Alpha".parse::<LastName>().expect("last name");
        assert_eq!(
            compare_persons(&a, &b, &PersonSort::PlaceOfResidence),
            Ordering::Less
        );
    }

    #[test]
    fn compare_created_at_and_updated_at_use_id_tiebreaker() {
        let mut a = base_person(1);
        let mut b = base_person(2);

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
