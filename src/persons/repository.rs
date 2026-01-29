use chrono::Utc;
use sqlx::PgPool;

use crate::{
    candidate_lists::CandidateListId,
    pagination::SortDirection,
    persons::{Gender, Person, PersonId, PersonSort},
};

pub async fn count_persons(db: &PgPool) -> Result<i64, sqlx::Error> {
    let record = sqlx::query!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM persons
        "#
    )
    .fetch_one(db)
    .await?;

    Ok(record.count)
}

pub async fn list_persons_not_on_candidate_list(
    db: &PgPool,
    candidate_list_id: CandidateListId,
) -> Result<Vec<Person>, sqlx::Error> {
    sqlx::query_file_as!(
        Person,
        "sql/persons/list_persons_not_on_candidate_list.sql",
        candidate_list_id.uuid(),
    )
    .fetch_all(db)
    .await
}

pub async fn list_persons(
    db: &PgPool,
    limit: i64,
    offset: i64,
    sort_field: &PersonSort,
    sort_direction: &SortDirection,
) -> Result<Vec<Person>, sqlx::Error> {
    sqlx::query_file_as!(
        Person,
        "sql/persons/list_persons.sql",
        limit,
        offset,
        sort_field.as_ref(),
        sort_direction.as_ref(),
    )
    .fetch_all(db)
    .await
}

pub async fn get_person(db: &PgPool, person_id: PersonId) -> Result<Option<Person>, sqlx::Error> {
    let person = sqlx::query_file_as!(Person, "sql/persons/get_person_by_id.sql", person_id.uuid())
        .fetch_optional(db)
        .await?;

    Ok(person)
}

pub async fn create_person(db: &PgPool, new_person: &Person) -> Result<Person, sqlx::Error> {
    sqlx::query_file_as!(
        Person,
        "sql/persons/insert_person.sql",
        new_person.id.uuid(),
        new_person.gender as Option<Gender>,
        new_person.last_name,
        new_person.last_name_prefix,
        new_person.first_name,
        new_person.initials,
        new_person.date_of_birth,
        new_person.bsn,
        new_person.place_of_residence,
        new_person.country_of_residence,
        new_person.locality,
        new_person.postal_code,
        new_person.house_number,
        new_person.house_number_addition,
        new_person.street_name,
        Utc::now(),
        Utc::now(),
    )
    .fetch_one(db)
    .await
}

pub async fn update_person(db: &PgPool, updated_person: &Person) -> Result<Person, sqlx::Error> {
    let person = sqlx::query_file_as!(
        Person,
        "sql/persons/update_person.sql",
        updated_person.gender as Option<Gender>,
        updated_person.last_name,
        updated_person.last_name_prefix,
        updated_person.first_name,
        updated_person.initials,
        updated_person.date_of_birth,
        updated_person.bsn,
        updated_person.place_of_residence,
        updated_person.country_of_residence,
        updated_person.id.uuid(),
    )
    .fetch_one(db)
    .await?;

    Ok(person)
}

pub async fn update_address(db: &PgPool, updated_person: &Person) -> Result<Person, sqlx::Error> {
    let person = sqlx::query_file_as!(
        Person,
        "sql/persons/update_address.sql",
        updated_person.locality,
        updated_person.postal_code,
        updated_person.house_number,
        updated_person.house_number_addition,
        updated_person.street_name,
        updated_person.id.uuid(),
    )
    .fetch_one(db)
    .await?;

    Ok(person)
}

pub async fn remove_person(db: &PgPool, person_id: PersonId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM persons
        WHERE id = $1
        "#,
        person_id.uuid(),
    )
    .execute(db)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    use crate::{
        candidate_lists,
        pagination::SortDirection,
        persons::PersonSort,
        test_utils::{sample_candidate_list, sample_person, sample_person_with_last_name},
    };

    #[sqlx::test]
    async fn create_and_get_person(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = PersonId::new();
        let person = sample_person(id);

        create_person(&pool, &person).await?;

        let loaded = get_person(&pool, id).await?.expect("person");
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.last_name, "Jansen");

        Ok(())
    }

    #[sqlx::test]
    async fn list_and_count_persons(pool: PgPool) -> Result<(), sqlx::Error> {
        create_person(
            &pool,
            &sample_person_with_last_name(PersonId::new(), "Jansen"),
        )
        .await?;
        create_person(
            &pool,
            &sample_person_with_last_name(PersonId::new(), "Bakker"),
        )
        .await?;

        let total = count_persons(&pool).await?;
        assert_eq!(total, 2);

        let persons =
            list_persons(&pool, 10, 0, &PersonSort::LastName, &SortDirection::Asc).await?;
        assert_eq!(persons.len(), 2);

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_overwrites_fields(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = PersonId::new();
        let mut person = sample_person(id);

        create_person(&pool, &person).await?;

        person.last_name = "Updated".to_string();
        update_person(&pool, &person).await?;

        let updated = get_person(&pool, id).await?.expect("person");
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn remove_person_deletes_record(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = PersonId::new();
        let person = sample_person(id);

        create_person(&pool, &person).await?;
        remove_person(&pool, id).await?;

        let missing = get_person(&pool, id).await?;
        assert!(missing.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn update_address_overwrites_fields(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = PersonId::new();
        let mut person = sample_person(id);

        create_person(&pool, &person).await?;

        person.locality = Some("Nieuwegein".to_string());
        person.postal_code = Some("9999 ZZ".to_string());
        person.house_number = Some("99".to_string());
        person.house_number_addition = None;
        person.street_name = Some("Nieuweweg".to_string());

        update_address(&pool, &person).await?;

        let updated = get_person(&pool, id).await?.expect("person");
        assert_eq!(updated.locality, Some("Nieuwegein".to_string()));
        assert_eq!(updated.postal_code, Some("9999 ZZ".to_string()));
        assert_eq!(updated.house_number, Some("99".to_string()));
        assert_eq!(updated.house_number_addition, None);
        assert_eq!(updated.street_name, Some("Nieuweweg".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn excludes_persons_on_candidate_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        candidate_lists::create_candidate_list(&pool, &list).await?;
        create_person(&pool, &person_a).await?;
        create_person(&pool, &person_b).await?;
        candidate_lists::update_candidate_list_order(&pool, list_id, &[person_a.id]).await?;

        let persons = list_persons_not_on_candidate_list(&pool, list_id).await?;
        assert_eq!(persons.len(), 1);
        assert_eq!(persons[0].id, person_b.id);

        Ok(())
    }
}
