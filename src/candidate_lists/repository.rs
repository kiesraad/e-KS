use sqlx::PgConnection;

use crate::{
    ElectoralDistrict,
    candidate_lists::{
        CandidateList, CandidateListId, CandidateListSummary,
    },
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use chrono::{Duration, Utc};
    use sqlx::PgPool;

    use crate::{
        candidate_lists, persons::{self, PersonId}, test_utils::{sample_candidate_list, sample_person_with_last_name}
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
        persons::create_person(&mut conn, &person_a).await?;
        candidate_lists::update_candidate_list_order(&mut conn, list_a.id, &[person_a.id]).await?;

        create_candidate_list(&mut conn, &list_b).await?;
        persons::create_person(&mut conn, &person_b).await?;
        candidate_lists::update_candidate_list_order(&mut conn, list_b.id, &[person_b.id]).await?;

        // test
        remove_candidate_list(&mut conn, list_a.id).await?;

        // verify
        let lists = list_candidate_list_with_count(&mut conn).await?;
        let list_b_from_db = candidate_lists::get_full_candidate_list(&mut conn, list_b.id)
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
