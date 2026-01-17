use sqlx::{Connection, PgConnection};

use crate::{
    ElectoralDistrict,
    candidate_lists::{
        Candidate, CandidateList, CandidateListId, CandidateListSummary, FullCandidateList,
    },
    persons::{Gender, Person, PersonId},
};

pub struct ListIdAndCount {
    pub id: CandidateListId,
    pub person_count: i64,
}

pub async fn list_candidate_list_with_count(
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

pub async fn list_candidate_list(
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

pub async fn get_candidate_list(
    conn: &mut PgConnection,
    list_id: CandidateListId,
) -> Result<Option<CandidateList>, sqlx::Error> {
    sqlx::query_as!(
        CandidateList,
        r#"
        SELECT id, electoral_districts AS "electoral_districts: Vec<ElectoralDistrict>", created_at, updated_at
        FROM candidate_lists
        WHERE id = $1
        "#,
        list_id.uuid(),
    )
    .fetch_optional(&mut *conn)
    .await
}

pub async fn get_full_candidate_list(
    conn: &mut PgConnection,
    list_id: CandidateListId,
) -> Result<Option<FullCandidateList>, sqlx::Error> {
    let list = get_candidate_list(conn, list_id).await?;

    let Some(list) = list else {
        return Ok(None);
    };

    let candidates = sqlx::query_file!(
        "sql/candidate_lists/list_candidates_for_list.sql",
        list.id.uuid()
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .map(|row| Candidate {
        list_id: list.id,
        position: row.position as usize,
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
pub async fn get_used_districts(
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

pub async fn create_candidate_list(
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
        candidate_list.id.uuid(),
        &candidate_list.electoral_districts as &[ElectoralDistrict],
        candidate_list.created_at,
        candidate_list.updated_at,
    )
    .fetch_one(conn)
    .await
}

pub async fn update_candidate_list_order(
    conn: &mut PgConnection,
    list_id: CandidateListId,
    person_ids: &[PersonId],
) -> Result<FullCandidateList, sqlx::Error> {
    let mut tx = conn.begin().await?;

    let updated = sqlx::query!(
        r#"
        UPDATE candidate_lists
        SET updated_at = NOW()
        WHERE id = $1
        "#,
        list_id.uuid(),
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
        list_id.uuid(),
    )
    .execute(&mut *tx)
    .await?;

    insert_candidates(&mut tx, list_id, person_ids).await?;

    tx.commit().await?;

    get_full_candidate_list(conn, list_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn append_candidate_to_list(
    conn: &mut PgConnection,
    list_id: CandidateListId,
    person_id: PersonId,
) -> Result<(), sqlx::Error> {
    let mut tx = conn.begin().await?;

    let updated = sqlx::query!(
        r#"
        UPDATE candidate_lists
        SET updated_at = NOW()
        WHERE id = $1
        "#,
        list_id.uuid(),
    )
    .execute(&mut *tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    let position_record = sqlx::query!(
        r#"
        SELECT COALESCE(MAX(position), 0) AS max_position
        FROM candidate_lists_persons
        WHERE candidate_list_id = $1
        "#,
        list_id.uuid(),
    )
    .fetch_one(&mut *tx)
    .await?;

    let new_position = position_record.max_position.unwrap_or(0) + 1;

    sqlx::query!(
        r#"
        INSERT INTO candidate_lists_persons (candidate_list_id, person_id, position)
        VALUES ($1, $2, $3)
        "#,
        list_id.uuid(),
        person_id.uuid(),
        new_position,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn remove_candidate(
    conn: &mut PgConnection,
    list_id: CandidateListId,
    person_id: PersonId,
) -> Result<(), sqlx::Error> {
    let mut tx = conn.begin().await?;

    let updated = sqlx::query!(
        r#"
        UPDATE candidate_lists
        SET updated_at = NOW()
        WHERE id = $1
        "#,
        list_id.uuid(),
    )
    .execute(&mut *tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    sqlx::query!(
        r#"
        DELETE FROM candidate_lists_persons
        WHERE candidate_list_id = $1 AND person_id = $2
        "#,
        list_id.uuid(),
        person_id.uuid(),
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn update_candidate_list(
    conn: &mut PgConnection,
    updated_candidate_list: CandidateList,
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
        updated_candidate_list.id.uuid()
    )
    .fetch_one(conn)
    .await
}

pub async fn remove_candidate_list(
    conn: &mut PgConnection,
    list_id: CandidateListId,
) -> Result<(), sqlx::Error> {
    // delete all the candidates first (otherwise we get a foreign key violation)
    sqlx::query!(
        r#"
        DELETE FROM candidate_lists_persons
        WHERE candidate_list_id = $1
        "#,
        list_id.uuid()
    )
    .execute(&mut *conn)
    .await?;

    // then, delete the list row itself
    sqlx::query!(
        r#"
        DELETE FROM candidate_lists
        WHERE id = $1
        "#,
        list_id.uuid()
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

async fn insert_candidates(
    executor: &mut PgConnection,
    list_id: CandidateListId,
    person_ids: &[PersonId],
) -> Result<(), sqlx::Error> {
    let positions: Vec<i32> = (1..=person_ids.len() as i32).collect();

    sqlx::query!(
        r#"
        INSERT INTO candidate_lists_persons (candidate_list_id, person_id, position)
        SELECT $1, person_id, position
        FROM UNNEST($2::uuid[], $3::int[]) AS t(person_id, position)
        "#,
        list_id.uuid(),
        &person_ids.iter().map(|p| p.uuid()).collect::<Vec<_>>(),
        &positions,
    )
    .execute(&mut *executor)
    .await?;

    Ok(())
}

pub async fn get_candidate(
    executor: &mut PgConnection,
    list_id: CandidateListId,
    person_id: PersonId,
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
        list_id.uuid(),
        person_id.uuid(),
    )
    .fetch_one(&mut *executor)
    .await?;

    Ok(Candidate {
        list_id,
        position: record.position as usize,
        person,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use chrono::{Duration, Utc};
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
            id: CandidateListId::new(),
            electoral_districts,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        create_candidate_list(conn, &list).await
    }

    #[sqlx::test]
    async fn create_and_list_candidate_lists(pool: PgPool) -> Result<(), sqlx::Error> {
        let list = sample_candidate_list(CandidateListId::new());

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list).await?;

        let lists = list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(1, lists.len());
        assert_eq!(list.id, lists[0].list.id);
        assert_eq!(0, lists[0].person_count);

        Ok(())
    }

    #[sqlx::test]
    async fn list_candidate_list_orders_by_created_at(pool: PgPool) -> Result<(), sqlx::Error> {
        let earlier = Utc::now() - Duration::days(1);
        let later = Utc::now();
        let list_early = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: earlier,
            updated_at: earlier,
        };
        let list_late = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::OV],
            created_at: later,
            updated_at: later,
        };

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list_late).await?;
        create_candidate_list(&mut conn, &list_early).await?;

        let lists = list_candidate_list(&mut conn).await?;
        assert_eq!(lists.len(), 2);
        assert_eq!(lists[0].id, list_early.id);
        assert_eq!(lists[1].id, list_late.id);

        Ok(())
    }

    #[sqlx::test]
    async fn get_candidate_list_returns_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let list = sample_candidate_list(CandidateListId::new());

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list).await?;

        let loaded = get_candidate_list(&mut conn, list.id)
            .await?
            .expect("candidate list");

        assert_eq!(loaded.id, list.id);

        Ok(())
    }

    #[sqlx::test]
    async fn get_candidate_list_includes_candidates(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person_a).await?;
        persons::repository::create_person(&mut conn, &person_b).await?;
        update_candidate_list_order(&mut conn, list_id, &[person_a.id, person_b.id]).await?;

        let detail = get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(2, detail.candidates.len());
        assert_eq!(person_a.id, detail.candidates[0].person.id);
        assert_eq!(person_b.id, detail.candidates[1].person.id);

        Ok(())
    }

    #[sqlx::test]
    async fn append_candidate_to_list_assigns_positions(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person_a).await?;
        persons::repository::create_person(&mut conn, &person_b).await?;

        append_candidate_to_list(&mut conn, list_id, person_a.id).await?;
        append_candidate_to_list(&mut conn, list_id, person_b.id).await?;

        let detail = get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");

        assert_eq!(detail.candidates.len(), 2);
        assert_eq!(detail.candidates[0].person.id, person_a.id);
        assert_eq!(detail.candidates[0].position, 1);
        assert_eq!(detail.candidates[1].person.id, person_b.id);
        assert_eq!(detail.candidates[1].position, 2);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_candidate_removes_from_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person_a).await?;
        persons::repository::create_person(&mut conn, &person_b).await?;
        append_candidate_to_list(&mut conn, list_id, person_a.id).await?;
        append_candidate_to_list(&mut conn, list_id, person_b.id).await?;

        remove_candidate(&mut conn, list_id, person_a.id).await?;

        let detail = get_full_candidate_list(&mut conn, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(detail.candidates.len(), 1);
        assert_eq!(detail.candidates[0].person.id, person_b.id);

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_list_updates_districts(pool: PgPool) -> Result<(), sqlx::Error> {
        let list = sample_candidate_list(CandidateListId::new());

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list).await?;

        let updated = update_candidate_list(
            &mut conn,
            CandidateList {
                electoral_districts: vec![ElectoralDistrict::DR, ElectoralDistrict::OV],
                ..list.clone()
            },
        )
        .await?;

        assert_eq!(updated.id, list.id);
        assert_eq!(
            updated.electoral_districts,
            vec![ElectoralDistrict::DR, ElectoralDistrict::OV]
        );

        Ok(())
    }

    #[sqlx::test]
    async fn get_candidate_returns_candidate(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Jansen");

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list).await?;
        persons::repository::create_person(&mut conn, &person).await?;
        append_candidate_to_list(&mut conn, list_id, person.id).await?;

        let candidate = get_candidate(&mut conn, list_id, person.id).await?;
        assert_eq!(candidate.list_id, list_id);
        assert_eq!(candidate.position, 1);
        assert_eq!(candidate.person.id, person.id);

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_list_returns_row_not_found(pool: PgPool) -> Result<(), sqlx::Error> {
        let mut conn = pool.acquire().await?;
        let err = update_candidate_list_order(&mut conn, CandidateListId::new(), &[])
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
        let list_a = sample_candidate_list(CandidateListId::new());
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let list_b = sample_candidate_list(CandidateListId::new());
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        create_candidate_list(&mut conn, &list_a).await?;
        persons::repository::create_person(&mut conn, &person_a).await?;
        update_candidate_list_order(&mut conn, list_a.id, &[person_a.id]).await?;

        create_candidate_list(&mut conn, &list_b).await?;
        persons::repository::create_person(&mut conn, &person_b).await?;
        update_candidate_list_order(&mut conn, list_b.id, &[person_b.id]).await?;

        // test
        remove_candidate_list(&mut conn, list_a.id).await?;

        // verify
        let lists = list_candidate_list_with_count(&mut conn).await?;
        let list_b_from_db = get_full_candidate_list(&mut conn, list_b.id)
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
