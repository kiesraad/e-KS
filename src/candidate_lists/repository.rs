use sqlx::{Connection, PgConnection};
use uuid::Uuid;

use crate::{
    ElectoralDistrict,
    candidate_lists::{Candidate, CandidateList, CandidateListSummary, FullCandidateList},
    persons::{Gender, Person},
};

pub struct ListIdAndCount {
    pub id: Uuid,
    pub person_count: i64,
}

pub(crate) async fn list_candidate_list_with_count(
    conn: &mut PgConnection,
) -> Result<Vec<CandidateListSummary>, sqlx::Error> {
    let counts = sqlx::query_as!(
        ListIdAndCount,
        r#"
            SELECT
                cl.id AS "id!",
                COUNT(clp.person_id)::bigint AS "person_count!"
            FROM candidate_lists cl
            LEFT JOIN candidate_lists_persons clp ON clp.candidate_list_id = cl.id
            GROUP BY cl.id
            ORDER BY cl.updated_at DESC, cl.created_at DESC
            "#,
    )
    .fetch_all(&mut *conn)
    .await?;

    let lists = list_candidate_list(conn).await?;

    Ok(lists
        .into_iter()
        .map(|list| {
            let person_count = counts
                .iter()
                .find(|c| c.id == list.id)
                .map(|c| c.person_count)
                .unwrap_or(0);

            CandidateListSummary { list, person_count }
        })
        .collect::<Vec<_>>())
}

pub(crate) async fn list_candidate_list(
    conn: &mut PgConnection,
) -> Result<Vec<CandidateList>, sqlx::Error> {
    sqlx::query_as!(
            CandidateList,
            r#"
            SELECT id, electoral_districts AS "electoral_districts: Vec<ElectoralDistrict>", created_at, updated_at
            FROM candidate_lists
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(conn)
        .await
}

pub(crate) async fn get_candidate_list(
    conn: &mut PgConnection,
    list_id: &Uuid,
) -> Result<Option<CandidateList>, sqlx::Error> {
    sqlx::query_as!(
        CandidateList,
        r#"
        SELECT id, electoral_districts AS "electoral_districts: Vec<ElectoralDistrict>", created_at, updated_at
        FROM candidate_lists
        WHERE id = $1
        "#,
        list_id,
    )
    .fetch_optional(&mut *conn)
    .await
}

pub(super) async fn get_full_candidate_list(
    conn: &mut PgConnection,
    list_id: &Uuid,
) -> Result<Option<FullCandidateList>, sqlx::Error> {
    let list = get_candidate_list(conn, list_id).await?;

    let Some(list) = list else {
        return Ok(None);
    };

    let candidates = sqlx::query!(
        r#"
        SELECT
            clp.position,
            p.id as "id!",
            p.gender as "gender?: Gender",
            p.last_name as "last_name!",
            p.last_name_prefix,
            p.first_name,
            p.initials as "initials!",
            p.date_of_birth,
            p.bsn,
            p.locality as "locality",
            p.postal_code as "postal_code",
            p.house_number as "house_number",
            p.house_number_addition,
            p.street_name as "street_name",
            p.is_dutch,
            p.custom_country,
            p.custom_region,
            p.address_line_1,
            p.address_line_2,
            p.created_at as "created_at!",
            p.updated_at as "updated_at!"
        FROM candidate_lists_persons clp
        JOIN persons p ON p.id = clp.person_id
        WHERE clp.candidate_list_id = $1
        ORDER BY clp.position ASC
        "#,
        list.id
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| Candidate {
        list_id: list.id,
        position: row.position,
        person: Person {
            id: row.id,
            gender: row.gender,
            last_name: row.last_name,
            last_name_prefix: row.last_name_prefix,
            first_name: row.first_name,
            initials: row.initials,
            date_of_birth: row.date_of_birth,
            bsn: row.bsn,
            locality: row.locality,
            postal_code: row.postal_code,
            house_number: row.house_number,
            house_number_addition: row.house_number_addition,
            street_name: row.street_name,
            is_dutch: row.is_dutch,
            custom_country: row.custom_country,
            custom_region: row.custom_region,
            address_line_1: row.address_line_1,
            address_line_2: row.address_line_2,
            created_at: row.created_at,
            updated_at: row.updated_at,
        },
    })
    .collect();

    Ok(Some(FullCandidateList { list, candidates }))
}

/// retrieves a vector of all the electoral districts that have been used in one or more candidate lists
pub(crate) async fn get_used_districts(
    conn: &mut PgConnection,
) -> Result<Vec<ElectoralDistrict>, sqlx::Error> {
    let districts = sqlx::query!(
        r#"
        SELECT array_agg(DISTINCT e) AS "electoral_districts: Vec<ElectoralDistrict>"
        FROM candidate_lists cl 
        CROSS JOIN LATERAL unnest(cl.electoral_districts ) AS e;
        "#
    )
    .fetch_one(&mut *conn)
    .await?
    .electoral_districts
    // if None is returned, there are no lists, so there are no used districts (empty set)
    .unwrap_or_default();
    Ok(districts)
}

pub(crate) async fn create_candidate_list(
    conn: &mut PgConnection,
    candidate_list: &CandidateList,
) -> Result<CandidateList, sqlx::Error> {
    sqlx::query_as!(
        CandidateList,
        r#"
        INSERT INTO candidate_lists (id, electoral_districts, created_at, updated_at)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            electoral_districts AS "electoral_districts: Vec<ElectoralDistrict>",
            created_at,
            updated_at
        "#,
        candidate_list.id,
        &candidate_list.electoral_districts as &[ElectoralDistrict],
        candidate_list.created_at,
        candidate_list.updated_at,
    )
    .fetch_one(conn)
    .await
}

pub(crate) async fn update_candidate_list_order(
    conn: &mut PgConnection,
    list_id: &Uuid,
    person_ids: &[Uuid],
) -> Result<FullCandidateList, sqlx::Error> {
    let mut tx = conn.begin().await?;

    let updated = sqlx::query!(
        r#"
        UPDATE candidate_lists
        SET updated_at = NOW()
        WHERE id = $1
        "#,
        list_id,
    )
    .execute(&mut *tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    sqlx::query!(
        r#"
        DELETE FROM candidate_lists_persons
        WHERE candidate_list_id = $1
        "#,
        list_id,
    )
    .execute(&mut *tx)
    .await?;

    insert_candidates(&mut tx, list_id, person_ids).await?;

    tx.commit().await?;

    get_full_candidate_list(conn, list_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub(crate) async fn update_candidate_list(
    conn: &mut PgConnection,
    updated_candidate_list: &CandidateList,
) -> Result<CandidateList, sqlx::Error> {
    sqlx::query_as!(
        CandidateList,
        r#"
        UPDATE candidate_lists
        SET
            electoral_districts = $1,
            updated_at = NOW()
        WHERE id = $2
        RETURNING
            id,
            electoral_districts AS "electoral_districts: Vec<ElectoralDistrict>",
            created_at,
            updated_at
        "#,
        &updated_candidate_list.electoral_districts as &[ElectoralDistrict],
        updated_candidate_list.id
    )
    .fetch_one(conn)
    .await
}

pub(crate) async fn remove_candidate_list(
    conn: &mut PgConnection,
    list_id: Uuid,
) -> Result<(), sqlx::Error> {
    // delete all the candidates first (otherwise we get a foreign key violation)
    sqlx::query!(
        r#"
        DELETE FROM candidate_lists_persons
        WHERE candidate_list_id = $1
        "#,
        list_id
    )
    .execute(&mut *conn)
    .await?;

    // then, delete the list row itself
    sqlx::query!(
        r#"
        DELETE FROM candidate_lists
        WHERE id = $1
        "#,
        list_id
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

async fn insert_candidates(
    executor: &mut PgConnection,
    list_id: &Uuid,
    person_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    let positions: Vec<i32> = (1..=person_ids.len() as i32).collect();

    sqlx::query!(
        r#"
        INSERT INTO candidate_lists_persons (candidate_list_id, person_id, position)
        SELECT $1, person_id, position
        FROM UNNEST($2::uuid[], $3::int[]) AS t(person_id, position)
        "#,
        list_id,
        &person_ids,
        &positions,
    )
    .execute(&mut *executor)
    .await?;

    Ok(())
}

pub async fn get_candidate(
    executor: &mut PgConnection,
    list_id: &Uuid,
    person_id: &Uuid,
) -> Result<Candidate, sqlx::Error> {
    let person = crate::persons::repository::get_person(executor, person_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    let record = sqlx::query!(
        r#"
        SELECT position
        FROM candidate_lists_persons
        WHERE candidate_list_id = $1 AND person_id = $2
        "#,
        list_id,
        person_id,
    )
    .fetch_one(&mut *executor)
    .await?;

    Ok(Candidate {
        list_id: *list_id,
        position: record.position,
        person,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use chrono::Utc;
    use sqlx::PgPool;

    use crate::{
        persons,
        test_utils::{sample_candidate_list, sample_person_with_last_name},
    };

    async fn insert_list(
        conn: &mut PgConnection,
        electoral_districts: Vec<ElectoralDistrict>,
    ) -> Result<CandidateList, sqlx::Error> {
        let list = CandidateList {
            id: Uuid::new_v4(),
            electoral_districts,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        create_candidate_list(conn, &list).await
    }

    #[sqlx::test]
    async fn create_and_list_candidate_lists(pool: PgPool) -> Result<(), sqlx::Error> {
        let list = sample_candidate_list(Uuid::new_v4());

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list).await?;

        let lists = list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(1, lists.len());
        assert_eq!(list.id, lists[0].list.id);
        assert_eq!(0, lists[0].person_count);

        Ok(())
    }

    #[sqlx::test]
    async fn get_candidate_list_includes_candidates(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(Uuid::new_v4(), "Jansen");
        let person_b = sample_person_with_last_name(Uuid::new_v4(), "Bakker");

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person_a).await?;
        persons::repository::create_person(&mut conn, &person_b).await?;
        update_candidate_list_order(&mut conn, &list_id, &[person_a.id, person_b.id]).await?;

        let detail = get_full_candidate_list(&mut conn, &list_id)
            .await?
            .expect("candidate list");
        assert_eq!(2, detail.candidates.len());
        assert_eq!(person_a.id, detail.candidates[0].person.id);
        assert_eq!(person_b.id, detail.candidates[1].person.id);

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_list_returns_row_not_found(pool: PgPool) -> Result<(), sqlx::Error> {
        let mut conn = pool.acquire().await?;
        let err = update_candidate_list_order(&mut conn, &Uuid::new_v4(), &[])
            .await
            .unwrap_err();
        assert!(matches!(err, sqlx::Error::RowNotFound));

        Ok(())
    }
    #[sqlx::test]
    async fn test_get_used_districts(pool: PgPool) -> Result<(), sqlx::Error> {
        // setup
        let mut conn = pool.acquire().await?;
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::OV,
        ]);

        insert_list(
            &mut conn,
            vec![ElectoralDistrict::UT, ElectoralDistrict::DR],
        )
        .await?;
        insert_list(&mut conn, vec![ElectoralDistrict::OV]).await?;
        insert_list(&mut conn, vec![]).await?;

        // test
        let result: BTreeSet<ElectoralDistrict> =
            get_used_districts(&mut conn).await?.into_iter().collect();

        // verify
        assert_eq!(expected, result);
        Ok(())
    }

    #[sqlx::test]
    async fn get_used_districts_no_lists(pool: PgPool) -> Result<(), sqlx::Error> {
        let mut conn = pool.acquire().await?;
        let result = get_used_districts(&mut conn).await?;

        assert_eq!(Vec::<ElectoralDistrict>::new(), result);

        Ok(())
    }

    #[sqlx::test]
    async fn get_used_districts_double_districts(pool: PgPool) -> Result<(), sqlx::Error> {
        let mut conn = pool.acquire().await?;
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::OV,
        ]);

        // setup
        insert_list(
            &mut conn,
            vec![ElectoralDistrict::UT, ElectoralDistrict::DR],
        )
        .await?;
        insert_list(
            &mut conn,
            vec![ElectoralDistrict::UT, ElectoralDistrict::OV],
        )
        .await?;

        // test
        let result: BTreeSet<ElectoralDistrict> =
            get_used_districts(&mut conn).await?.into_iter().collect();

        // verify
        assert_eq!(expected, result);
        Ok(())
    }

    #[sqlx::test]
    async fn test_remove_candidate_list(pool: PgPool) -> Result<(), sqlx::Error> {
        // setup
        let mut conn = pool.acquire().await?;
        let list_a = sample_candidate_list(Uuid::new_v4());
        let person_a = sample_person_with_last_name(Uuid::new_v4(), "Jansen");
        let list_b = sample_candidate_list(Uuid::new_v4());
        let person_b = sample_person_with_last_name(Uuid::new_v4(), "Bakker");

        create_candidate_list(&mut conn, &list_a).await?;
        persons::repository::create_person(&mut conn, &person_a).await?;
        update_candidate_list_order(&mut conn, &list_a.id, &[person_a.id]).await?;

        create_candidate_list(&mut conn, &list_b).await?;
        persons::repository::create_person(&mut conn, &person_b).await?;
        update_candidate_list_order(&mut conn, &list_b.id, &[person_b.id]).await?;

        // test
        remove_candidate_list(&mut conn, list_a.id).await?;

        // verify
        let lists = list_candidate_list_with_count(&mut conn).await?;
        let list_b_from_db = get_full_candidate_list(&mut conn, &list_b.id)
            .await?
            .unwrap();
        // one list remains
        assert_eq!(1, lists.len());
        // the correct list got deleted
        assert_eq!(list_b.id, lists[0].list.id);
        // and only persons got removed associated with the deleted list
        assert_eq!(1, lists[0].person_count);
        assert_eq!(person_b.id, list_b_from_db.candidates[0].person.id);

        Ok(())
    }
}
