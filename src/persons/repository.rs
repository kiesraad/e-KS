use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    pagination::SortDirection,
    persons::structs::{Gender, Person, PersonSort},
};

pub(crate) async fn count_persons(conn: &mut PgConnection) -> Result<i64, sqlx::Error> {
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

pub(crate) async fn list_persons_not_on_candidate_list(
    conn: &mut PgConnection,
    candidate_list_id: &Uuid,
) -> Result<Vec<Person>, sqlx::Error> {
    let persons = sqlx::query_as!(
        Person,
        r#"
        SELECT
            id,
            gender as "gender?: Gender",
            last_name,
            last_name_prefix,
            first_name,
            initials,
            date_of_birth,
            bsn,
            locality,
            postal_code,
            house_number,
            house_number_addition,
            street_name,
            is_dutch,
            custom_country,
            custom_region,
            address_line_1,
            address_line_2,
            created_at,
            updated_at
        FROM persons
        WHERE id NOT IN (
            SELECT person_id
            FROM candidate_lists_persons
            WHERE candidate_list_id = $1
        )
        ORDER BY last_name asc, initials asc
        "#,
        candidate_list_id,
    )
    .fetch_all(conn)
    .await?;

    Ok(persons)
}

pub(crate) async fn list_persons(
    conn: &mut PgConnection,
    limit: i64,
    offset: i64,
    sort_field: &PersonSort,
    sort_direction: &SortDirection,
) -> Result<Vec<Person>, sqlx::Error> {
    let persons = sqlx::query_as!(
        Person,
        r#"
        SELECT
            id,
            gender as "gender?: Gender",
            last_name,
            last_name_prefix,
            first_name,
            initials,
            date_of_birth,
            bsn,
            locality,
            postal_code,
            house_number,
            house_number_addition,
            street_name,
            address_line_1,
            address_line_2,
            is_dutch,
            custom_country,
            custom_region,
            created_at,
            updated_at
        FROM persons
        ORDER BY
            CASE WHEN $3 = 'last_name' AND $4 = 'asc' THEN last_name END ASC,
            CASE WHEN $3 = 'last_name' AND $4 = 'desc' THEN last_name END DESC,
            CASE WHEN $3 = 'first_name' AND $4 = 'asc' THEN first_name END ASC,
            CASE WHEN $3 = 'first_name' AND $4 = 'desc' THEN first_name END DESC,
            CASE WHEN $3 = 'initials' AND $4 = 'asc' THEN initials END ASC,
            CASE WHEN $3 = 'initials' AND $4 = 'desc' THEN initials END DESC,
            CASE WHEN $3 = 'gender' AND $4 = 'asc' THEN gender END ASC,
            CASE WHEN $3 = 'gender' AND $4 = 'desc' THEN gender END DESC,
            CASE WHEN $3 = 'locality' AND $4 = 'asc' THEN locality END ASC,
            CASE WHEN $3 = 'locality' AND $4 = 'desc' THEN locality END DESC,
            CASE WHEN $3 = 'created_at' AND $4 = 'asc' THEN created_at END ASC,
            CASE WHEN $3 = 'created_at' AND $4 = 'desc' THEN created_at END DESC,
            CASE WHEN $3 = 'updated_at' AND $4 = 'asc' THEN updated_at END ASC,
            CASE WHEN $3 = 'updated_at' AND $4 = 'desc' THEN updated_at END DESC,
            id DESC
        LIMIT $1
        OFFSET $2
        "#,
        limit,
        offset,
        sort_field.as_ref(),
        sort_direction.as_ref(),
    )
    .fetch_all(conn)
    .await?;

    Ok(persons)
}

pub(crate) async fn get_person(
    conn: &mut PgConnection,
    person_id: &Uuid,
) -> Result<Option<Person>, sqlx::Error> {
    let person = sqlx::query_as!(
        Person,
        r#"
        SELECT
            id,
            gender as "gender?: Gender",
            last_name,
            last_name_prefix,
            first_name,
            initials,
            date_of_birth,
            bsn,
            locality,
            postal_code,
            house_number,
            house_number_addition,
            street_name,
            is_dutch,
            custom_country,
            custom_region,
            address_line_1,
            address_line_2,
            created_at,
            updated_at
        FROM persons
        WHERE id = $1
        "#,
        person_id,
    )
    .fetch_optional(conn)
    .await?;

    Ok(person)
}

pub(crate) async fn create_person(
    conn: &mut PgConnection,
    new_person: &Person,
) -> Result<Person, sqlx::Error> {
    sqlx::query_as!(
        Person,
        r#"
        INSERT INTO persons (
            id,
            gender,
            last_name,
            last_name_prefix,
            first_name,
            initials,
            date_of_birth,
            bsn,
            locality,
            postal_code,
            house_number,
            house_number_addition,
            street_name,
            is_dutch,
            custom_country,
            custom_region,
            address_line_1,
            address_line_2,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        RETURNING
            id,
            gender as "gender?: Gender",
            last_name,
            last_name_prefix,
            first_name,
            initials,
            date_of_birth,
            bsn,
            locality,
            postal_code,
            house_number,
            house_number_addition,
            street_name,
            is_dutch,
            custom_country,
            custom_region,
            address_line_1,
            address_line_2,
            created_at,
            updated_at
        "#,
        new_person.id,
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

pub(crate) async fn update_person(
    conn: &mut PgConnection,
    updated_person: &Person,
) -> Result<Person, sqlx::Error> {
    let person = sqlx::query_as!(
        Person,
        r#"
        UPDATE persons
        SET
            gender = $1,
            last_name = $2,
            last_name_prefix = $3,
            first_name = $4,
            initials = $5,
            date_of_birth = $6,
            bsn = $7,
            locality = $8,
            postal_code = $9,
            house_number = $10,
            house_number_addition = $11,
            street_name = $12,
            is_dutch = $13,
            custom_country = $14,
            custom_region = $15,
            address_line_1 = $16,
            address_line_2 = $17,
            updated_at = NOW()
        WHERE id = $18
        RETURNING
            id,
            gender as "gender?: Gender",
            last_name,
            last_name_prefix,
            first_name,
            initials,
            date_of_birth,
            bsn,
            locality,
            postal_code,
            house_number,
            house_number_addition,
            street_name,
            is_dutch,
            custom_country,
            custom_region,
            address_line_1,
            address_line_2,
            created_at,
            updated_at
        "#,
        updated_person.gender as Option<Gender>,
        updated_person.last_name,
        updated_person.last_name_prefix,
        updated_person.first_name,
        updated_person.initials,
        updated_person.date_of_birth,
        updated_person.bsn,
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
        updated_person.id,
    )
    .fetch_one(conn)
    .await?;

    Ok(person)
}

pub(super) async fn remove_person(
    conn: &mut PgConnection,
    person_id: &Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM persons
        WHERE id = $1
        "#,
        person_id,
    )
    .execute(conn)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Utc};
    use sqlx::PgPool;

    use crate::{
        ElectoralDistrict,
        candidate_lists::{repository as candidate_lists_repository, structs::CandidateList},
        pagination::SortDirection,
        persons::structs::PersonSort,
    };

    fn sample_person(id: Uuid, last_name: &str) -> Person {
        Person {
            id,
            gender: Some(Gender::Female),
            last_name: last_name.to_string(),
            last_name_prefix: None,
            first_name: Some("Marlon".to_string()),
            initials: "M.B.".to_string(),
            date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 2, 1).unwrap()),
            bsn: None,
            locality: Some("Utrecht".to_string()),
            postal_code: Some("1234 AB".to_string()),
            house_number: Some("10".to_string()),
            house_number_addition: Some("A".to_string()),
            street_name: Some("Stationsstraat".to_string()),
            is_dutch: Some(true),
            custom_country: None,
            custom_region: None,
            address_line_1: None,
            address_line_2: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_list(id: Uuid) -> CandidateList {
        CandidateList {
            id,
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[sqlx::test]
    async fn create_and_get_person(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let person = sample_person(id, "Doe");

        let mut conn = pool.acquire().await?;
        create_person(&mut conn, &person).await?;

        let loaded = get_person(&mut conn, &id).await?.expect("person");
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.last_name, "Doe");

        Ok(())
    }

    #[sqlx::test]
    async fn list_and_count_persons(pool: PgPool) -> Result<(), sqlx::Error> {
        let mut conn = pool.acquire().await?;
        create_person(&mut conn, &sample_person(Uuid::new_v4(), "Alpha")).await?;
        create_person(&mut conn, &sample_person(Uuid::new_v4(), "Beta")).await?;

        let total = count_persons(&mut conn).await?;
        assert_eq!(total, 2);

        let persons =
            list_persons(&mut conn, 10, 0, &PersonSort::LastName, &SortDirection::Asc).await?;
        assert_eq!(persons.len(), 2);

        Ok(())
    }

    #[sqlx::test]
    async fn update_person_overwrites_fields(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let mut person = sample_person(id, "Doe");

        let mut conn = pool.acquire().await?;
        create_person(&mut conn, &person).await?;

        person.last_name = "Updated".to_string();
        update_person(&mut conn, &person).await?;

        let updated = get_person(&mut conn, &id).await?.expect("person");
        assert_eq!(updated.last_name, "Updated");

        Ok(())
    }

    #[sqlx::test]
    async fn remove_person_deletes_record(pool: PgPool) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4();
        let person = sample_person(id, "Doe");

        let mut conn = pool.acquire().await?;
        create_person(&mut conn, &person).await?;
        remove_person(&mut conn, &id).await?;

        let missing = get_person(&mut conn, &id).await?;
        assert!(missing.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn excludes_persons_on_candidate_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_list(list_id);
        let person_a = sample_person(Uuid::new_v4(), "Alpha");
        let person_b = sample_person(Uuid::new_v4(), "Beta");

        let mut conn = pool.acquire().await?;
        candidate_lists_repository::create_candidate_list(&mut conn, &list).await?;
        create_person(&mut conn, &person_a).await?;
        create_person(&mut conn, &person_b).await?;
        candidate_lists_repository::update_candidate_list(&mut conn, &list_id, &[person_a.id])
            .await?;

        let persons = list_persons_not_on_candidate_list(&mut conn, &list_id).await?;
        assert_eq!(persons.len(), 1);
        assert_eq!(persons[0].id, person_b.id);

        Ok(())
    }
}
