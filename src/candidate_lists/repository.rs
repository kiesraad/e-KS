use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    ElectoralDistrict,
    candidate_lists::{CandidateList, CandidateListId, CandidateListSummary},
};

pub struct ListIdAndCount {
    pub id: CandidateListId,
    pub person_count: i64,
}

pub async fn list_candidate_list_summary(
    db: &PgPool,
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
    .fetch_all(db)
    .await?;

    let lists = list_candidate_list(db).await?;

    let mut summaries = vec![];

    for list in lists {
        let person_count = counts
            .iter()
            .find(|c| c.id == list.id)
            .map(|c| c.person_count)
            .unwrap_or(0);
        let duplicate_districts = get_duplicate_districts(db, &list).await?;

        summaries.push(CandidateListSummary {
            list,
            person_count,
            duplicate_districts,
        });
    }

    Ok(summaries)
}

async fn get_duplicate_districts(
    db: &PgPool,
    list: &CandidateList,
) -> Result<Vec<ElectoralDistrict>, sqlx::Error> {
    let used_districts = get_used_districts(db, vec![list.id]).await?;

    Ok(list
        .electoral_districts
        .clone()
        .into_iter()
        .filter(|district| used_districts.contains(district))
        .collect())
}

pub async fn list_candidate_list(db: &PgPool) -> Result<Vec<CandidateList>, sqlx::Error> {
    sqlx::query_as!(
        CandidateList,
        r#"
            SELECT id, electoral_districts AS "electoral_districts: Vec<ElectoralDistrict>", created_at, updated_at
            FROM candidate_lists
            ORDER BY created_at ASC
            "#,
    )
    .fetch_all(db)
    .await
}

pub async fn get_candidate_list(
    db: &PgPool,
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
    .fetch_optional(db)
    .await
}

/// Retrieves a vector of all the electoral districts that have been used in one or more candidate lists.
/// Optionally, include list ids to exclude in the aggregation
pub async fn get_used_districts(
    db: &PgPool,
    exclude_list_ids: Vec<CandidateListId>,
) -> Result<Vec<ElectoralDistrict>, sqlx::Error> {
    let ids: Vec<Uuid> = exclude_list_ids.iter().map(|list| list.uuid()).collect();

    let districts = sqlx::query!(
        r#"
        SELECT array_agg(DISTINCT e) AS "electoral_districts: Vec<ElectoralDistrict>"
        FROM candidate_lists cl 
        CROSS JOIN LATERAL unnest(cl.electoral_districts ) AS e
        WHERE cl.id != ALL($1);
        "#,
        &ids
    )
    .fetch_one(db)
    .await?
    .electoral_districts
    // if None is returned, there are no lists, so there are no used districts (empty set)
    .unwrap_or_default();
    Ok(districts)
}

pub async fn create_candidate_list(
    db: &PgPool,
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
        Utc::now(),
        Utc::now(),
    )
    .fetch_one(db)
    .await
}

pub async fn update_candidate_list(
    db: &PgPool,
    updated_candidate_list: &CandidateList,
) -> Result<CandidateList, sqlx::Error> {
    sqlx::query_as!(
        CandidateList,
        r#"
        UPDATE candidate_lists
        SET
            electoral_districts = $1,
            updated_at = $3
        WHERE id = $2
        RETURNING
            id,
            electoral_districts AS "electoral_districts: Vec<ElectoralDistrict>",
            created_at,
            updated_at
        "#,
        &updated_candidate_list.electoral_districts as &[ElectoralDistrict],
        updated_candidate_list.id.uuid(),
        Utc::now(),
    )
    .fetch_one(db)
    .await
}

pub async fn remove_candidate_list(
    db: &PgPool,
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
    .execute(db)
    .await?;

    // then, delete the list row itself
    sqlx::query!(
        r#"
        DELETE FROM candidate_lists
        WHERE id = $1
        "#,
        list_id.uuid()
    )
    .execute(db)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::collections::BTreeSet;

    use super::*;
    use chrono::Utc;
    use sqlx::PgPool;

    use crate::{
        candidate_lists,
        persons::{self, PersonId},
        test_utils::{sample_candidate_list, sample_person_with_last_name},
    };

    async fn insert_list(
        db: &PgPool,
        electoral_districts: Vec<ElectoralDistrict>,
    ) -> Result<CandidateList, sqlx::Error> {
        let list = CandidateList {
            id: CandidateListId::new(),
            electoral_districts,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        create_candidate_list(db, &list).await
    }

    #[sqlx::test]
    async fn create_and_list_candidate_lists(pool: PgPool) -> Result<(), sqlx::Error> {
        let list = sample_candidate_list(CandidateListId::new());

        create_candidate_list(&pool, &list).await?;

        let lists = list_candidate_list_summary(&pool).await?;
        assert_eq!(1, lists.len());
        assert_eq!(list.id, lists[0].list.id);
        assert_eq!(0, lists[0].person_count);
        assert_eq!(0, lists[0].duplicate_districts.len());

        Ok(())
    }

    #[sqlx::test]
    async fn get_candidate_list_summaries_with_duplicate_districts(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        // setup
        let list1 = insert_list(&pool, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        let list2 = insert_list(&pool, vec![ElectoralDistrict::UT, ElectoralDistrict::GR]).await?;

        let list3 = insert_list(&pool, vec![ElectoralDistrict::OV, ElectoralDistrict::GR]).await?;

        // test
        let lists = list_candidate_list_summary(&pool).await?;

        // verification
        assert_eq!(3, lists.len());

        let list_summary1 = lists.iter().find(|list| list.list.id == list1.id).unwrap();
        let list_summary2 = lists.iter().find(|list| list.list.id == list2.id).unwrap();
        let list_summary3 = lists.iter().find(|list| list.list.id == list3.id).unwrap();

        // list 1 clashes on UT with list 2
        assert_eq!(
            vec![ElectoralDistrict::UT],
            list_summary1.duplicate_districts
        );

        // list 2 clashes on UT with list 1 and on GR with list 3
        assert_eq!(2, list_summary2.duplicate_districts.len());
        assert!(
            list_summary2
                .duplicate_districts
                .contains(&ElectoralDistrict::UT)
        );
        assert!(
            list_summary2
                .duplicate_districts
                .contains(&ElectoralDistrict::GR)
        );

        // list 3 clashes on GR with list 2
        assert_eq!(
            vec![ElectoralDistrict::GR],
            list_summary3.duplicate_districts
        );

        Ok(())
    }

    #[sqlx::test]
    async fn list_candidate_list_orders_by_created_at(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_early = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let list_late = CandidateList {
            id: CandidateListId::new(),
            electoral_districts: vec![ElectoralDistrict::OV],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        create_candidate_list(&pool, &list_early).await?;
        tokio::time::sleep(time::Duration::from_millis(10)).await;
        create_candidate_list(&pool, &list_late).await?;

        let lists = list_candidate_list(&pool).await?;
        assert_eq!(lists.len(), 2);
        assert_eq!(lists[0].id, list_early.id);
        assert_eq!(lists[1].id, list_late.id);

        Ok(())
    }

    #[sqlx::test]
    async fn get_candidate_list_returns_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let list = sample_candidate_list(CandidateListId::new());

        create_candidate_list(&pool, &list).await?;

        let loaded = get_candidate_list(&pool, list.id)
            .await?
            .expect("candidate list");

        assert_eq!(loaded.id, list.id);

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_list_updates_districts(pool: PgPool) -> Result<(), sqlx::Error> {
        let list = sample_candidate_list(CandidateListId::new());

        create_candidate_list(&pool, &list).await?;

        let updated = update_candidate_list(
            &pool,
            &CandidateList {
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
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::OV,
        ]);

        insert_list(&pool, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        insert_list(&pool, vec![ElectoralDistrict::OV]).await?;
        insert_list(&pool, vec![]).await?;

        // test
        let result: BTreeSet<ElectoralDistrict> = get_used_districts(&pool, vec![])
            .await?
            .into_iter()
            .collect();

        // verify
        assert_eq!(expected, result);
        Ok(())
    }

    #[sqlx::test]
    async fn get_used_districts_no_lists(pool: PgPool) -> Result<(), sqlx::Error> {
        let result = get_used_districts(&pool, vec![]).await?;

        assert_eq!(Vec::<ElectoralDistrict>::new(), result);

        Ok(())
    }

    #[sqlx::test]
    async fn get_used_districts_double_districts(pool: PgPool) -> Result<(), sqlx::Error> {
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::OV,
        ]);

        // setup
        insert_list(&pool, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        insert_list(&pool, vec![ElectoralDistrict::UT, ElectoralDistrict::OV]).await?;

        // test
        let result: BTreeSet<ElectoralDistrict> = get_used_districts(&pool, vec![])
            .await?
            .into_iter()
            .collect();

        // verify
        assert_eq!(expected, result);
        Ok(())
    }

    #[sqlx::test]
    async fn get_used_district_with_exclude(pool: PgPool) -> Result<(), sqlx::Error> {
        let expected = BTreeSet::from([
            ElectoralDistrict::UT,
            ElectoralDistrict::DR,
            ElectoralDistrict::GR,
            ElectoralDistrict::OV,
        ]);

        // setup
        insert_list(&pool, vec![ElectoralDistrict::UT, ElectoralDistrict::DR]).await?;
        insert_list(&pool, vec![ElectoralDistrict::GR, ElectoralDistrict::OV]).await?;

        let exclude_id = insert_list(&pool, vec![ElectoralDistrict::GR, ElectoralDistrict::LI])
            .await?
            .id;

        // test
        let result: BTreeSet<ElectoralDistrict> = get_used_districts(&pool, vec![exclude_id])
            .await?
            .into_iter()
            .collect();

        // verify
        assert_eq!(expected, result);

        Ok(())
    }

    #[sqlx::test]
    async fn test_remove_candidate_list(pool: PgPool) -> Result<(), sqlx::Error> {
        // setup
        let list_a = sample_candidate_list(CandidateListId::new());
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let list_b = sample_candidate_list(CandidateListId::new());
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        create_candidate_list(&pool, &list_a).await?;
        persons::create_person(&pool, &person_a).await?;
        candidate_lists::update_candidate_list_order(&pool, list_a.id, &[person_a.id]).await?;

        create_candidate_list(&pool, &list_b).await?;
        persons::create_person(&pool, &person_b).await?;
        candidate_lists::update_candidate_list_order(&pool, list_b.id, &[person_b.id]).await?;

        // test
        remove_candidate_list(&pool, list_a.id).await?;

        // verify
        let lists = list_candidate_list_summary(&pool).await?;
        let list_b_from_db = candidate_lists::get_full_candidate_list(&pool, list_b.id)
            .await?
            .unwrap();
        // one list remains
        assert_eq!(1, lists.len());
        // the correct list got deleted
        assert_eq!(list_b.id, lists[0].list.id);
        // and only persons got removed associated with the deleted list
        assert_eq!(1, lists[0].person_count);
        assert_eq!(person_b.id, list_b_from_db.candidates[0].person.id);
        // no duplicate districts
        assert_eq!(0, lists[0].duplicate_districts.len());

        Ok(())
    }
}
