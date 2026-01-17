use sqlx::PgConnection;

use crate::{
    candidate_lists::CandidateListId,
    pagination::SortDirection,
    persons::{Gender, Person, PersonId, PersonSort},
};

pub async fn count_persons(conn: &mut PgConnection) -> Result<i64, sqlx::Error> {
    let record = sqlx::query!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM persons
        "#
    )
    .fetch_one(conn)
    .await?;

    Ok(record.count)
}

pub async fn list_persons_not_on_candidate_list(
    conn: &mut PgConnection,
    candidate_list_id: CandidateListId,
) -> Result<Vec<Person>, sqlx::Error> {
    sqlx::query_file_as!(
        Person,
        "sql/persons/list_persons_not_on_candidate_list.sql",
        candidate_list_id.uuid(),
    )
    .fetch_all(conn)
    .await
}

pub async fn list_persons(
    conn: &mut PgConnection,
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
    .fetch_all(conn)
    .await
}

#[cfg(feature = "fixtures")]
pub async fn list_all_persons(conn: &mut PgConnection) -> Result<Vec<Person>, sqlx::Error> {
    sqlx::query_file_as!(Person, "sql/persons/list_all_persons.sql")
        .fetch_all(conn)
        .await
}

pub async fn get_person(
    conn: &mut PgConnection,
    person_id: PersonId,
) -> Result<Option<Person>, sqlx::Error> {
    let person = sqlx::query_file_as!(Person, "sql/persons/get_person_by_id.sql", person_id.uuid())
        .fetch_optional(conn)
        .await?;

    Ok(person)
}

pub async fn create_person(
    conn: &mut PgConnection,
    new_person: &Person,
) -> Result<Person, sqlx::Error> {
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
        new_person.locality,
        new_person.postal_code,
        new_person.house_number,
        new_person.house_number_addition,
        new_person.street_name,
        new_person.is_dutch,
        new_person.custom_country,
        new_person.custom_region,
        new_person.address_line_1,
        new_person.address_line_2,
        new_person.created_at,
        new_person.updated_at,
    )
    .fetch_one(conn)
    .await
}

pub async fn update_person(
    conn: &mut PgConnection,
    updated_person: &Person,
) -> Result<Person, sqlx::Error> {
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
        updated_person.id.uuid(),
    )
    .fetch_one(conn)
    .await?;

    Ok(person)
}

pub async fn update_address(
    conn: &mut PgConnection,
    updated_person: &Person,
) -> Result<Person, sqlx::Error> {
    let person = sqlx::query_file_as!(
        Person,
        "sql/persons/update_address.sql",
        updated_person.locality,
        updated_person.postal_code,
        updated_person.house_number,
        updated_person.house_number_addition,
        updated_person.street_name,
        updated_person.is_dutch,
        updated_person.custom_country,
        updated_person.custom_region,
        updated_person.address_line_1,
        updated_person.address_line_2,
        updated_person.id.uuid(),
    )
    .fetch_one(conn)
    .await?;

    Ok(person)
}

pub async fn remove_person(
    conn: &mut PgConnection,
    person_id: PersonId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM persons
        WHERE id = $1
        "#,
        person_id.uuid(),
    )
    .execute(conn)
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

        let mut conn = pool.acquire().await?;
        create_person(&mut conn, &person).await?;

        let loaded = get_person(&mut conn, id).await?.expect("person");
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.last_name, "Jansen");

        Ok(())
    }

    #[sqlx::test]
    async fn list_and_count_persons(pool: PgPool) -> Result<(), sqlx::Error> {
        let mut conn = pool.acquire().await?;
        create_person(
            &mut conn,
            &sample_person_with_last_name(PersonId::new(), "Jansen"),
        )
        .await?;
        create_person(
            &mut conn,
            &sample_person_with_last_name(PersonId::new(), "Bakker"),
        )
        .await?;

        let total = count_persons(&mut conn).await?;
        assert_eq!(total, 2);

        let persons =
            list_persons(&mut conn, 10, 0, &PersonSort::LastName, &SortDirection::Asc).await?;
        assert_eq!(persons.len(), 2);

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_overwrites_fields(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = PersonId::new();
        let mut person = sample_person(id);

        let mut conn = pool.acquire().await?;
        create_person(&mut conn, &person).await?;

        person.last_name = "Updated".to_string();
        update_person(&mut conn, &person).await?;

        let updated = get_person(&mut conn, id).await?.expect("person");
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn remove_person_deletes_record(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = PersonId::new();
        let person = sample_person(id);

        let mut conn = pool.acquire().await?;
        create_person(&mut conn, &person).await?;
        remove_person(&mut conn, id).await?;

        let missing = get_person(&mut conn, id).await?;
        assert!(missing.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn excludes_persons_on_candidate_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        let mut conn = pool.acquire().await?;
        candidate_lists::repository::create_candidate_list(&mut conn, &list).await?;
        create_person(&mut conn, &person_a).await?;
        create_person(&mut conn, &person_b).await?;
        candidate_lists::repository::update_candidate_list_order(
            &mut conn,
            list_id,
            &[person_a.id],
        )
        .await?;

        let persons = list_persons_not_on_candidate_list(&mut conn, list_id).await?;
        assert_eq!(persons.len(), 1);
        assert_eq!(persons[0].id, person_b.id);

        Ok(())
    }
}
