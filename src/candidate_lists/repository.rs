use sqlx::{Connection, PgConnection};
use uuid::Uuid;

use crate::{
    ElectoralDistrict,
    candidate_lists::structs::MAX_CANDIDATES,
    persons::structs::{Gender, Person},
};

use super::structs::{
    CandidateList, CandidateListDetail, CandidateListEntry, CandidateListSummary,
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
            ORDER BY updated_at DESC, created_at DESC
            "#,
        )
        .fetch_all(conn)
        .await
}

pub(super) async fn get_candidate_list(
    conn: &mut PgConnection,
    list_id: &Uuid,
) -> Result<Option<CandidateListDetail>, sqlx::Error> {
    let list = sqlx::query_as!(
        CandidateList,
        r#"
        SELECT id, electoral_districts AS "electoral_districts: Vec<ElectoralDistrict>", created_at, updated_at
        FROM candidate_lists
        WHERE id = $1
        "#,
        list_id,
    )
    .fetch_optional(&mut *conn)
    .await?;

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
            p.first_name,
            p.initials as "initials!",
            p.date_of_birth,
            p.locality as "locality",
            p.postal_code as "postal_code",
            p.house_number as "house_number",
            p.house_number_addition,
            p.street_name as "street_name",
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
    .map(|row| CandidateListEntry {
        position: row.position,
        person: Person {
            id: row.id,
            gender: row.gender,
            last_name: row.last_name,
            first_name: row.first_name,
            initials: row.initials,
            date_of_birth: row.date_of_birth,
            locality: row.locality,
            postal_code: row.postal_code,
            house_number: row.house_number,
            house_number_addition: row.house_number_addition,
            street_name: row.street_name,
            created_at: row.created_at,
            updated_at: row.updated_at,
        },
    })
    .collect();

    Ok(Some(CandidateListDetail { list, candidates }))
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

pub(crate) async fn update_candidate_list(
    conn: &mut PgConnection,
    list_id: &Uuid,
    person_ids: &[Uuid],
) -> Result<CandidateListDetail, sqlx::Error> {
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

    get_candidate_list(conn, list_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

async fn insert_candidates(
    executor: &mut PgConnection,
    list_id: &Uuid,
    person_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    let limited_ids: Vec<Uuid> = person_ids.iter().copied().take(MAX_CANDIDATES).collect();
    if limited_ids.is_empty() {
        return Ok(());
    }

    let positions: Vec<i32> = (1..=limited_ids.len() as i32).collect();

    sqlx::query!(
        r#"
        INSERT INTO candidate_lists_persons (candidate_list_id, person_id, position)
        SELECT $1, person_id, position
        FROM UNNEST($2::uuid[], $3::int[]) AS t(person_id, position)
        "#,
        list_id,
        &limited_ids,
        &positions,
    )
    .execute(&mut *executor)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Utc};
    use sqlx::PgPool;

    use crate::persons::repository as persons_repository;

    fn sample_list(id: Uuid) -> CandidateList {
        CandidateList {
            id,
            electoral_districts: vec![ElectoralDistrict::UT],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_person(id: Uuid, last_name: &str) -> Person {
        Person {
            id,
            gender: Some(Gender::Female),
            last_name: last_name.to_string(),
            first_name: Some("Marlon".to_string()),
            initials: "M.B.".to_string(),
            date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 2, 1).unwrap()),
            locality: Some("Utrecht".to_string()),
            postal_code: Some("1234 AB".to_string()),
            house_number: Some("10".to_string()),
            house_number_addition: Some("A".to_string()),
            street_name: Some("Stationsstraat".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[sqlx::test]
    async fn create_and_list_candidate_lists(pool: PgPool) -> Result<(), sqlx::Error> {
        let list = sample_list(Uuid::new_v4());

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list).await?;

        let lists = list_candidate_list_with_count(&mut conn).await?;
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].list.id, list.id);
        assert_eq!(lists[0].person_count, 0);

        Ok(())
    }

    #[sqlx::test]
    async fn get_candidate_list_includes_candidates(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = Uuid::new_v4();
        let list = sample_list(list_id);
        let person_a = sample_person(Uuid::new_v4(), "Alpha");
        let person_b = sample_person(Uuid::new_v4(), "Beta");

        let mut conn = pool.acquire().await?;
        create_candidate_list(&mut conn, &list).await?;
        persons_repository::create_person(&mut conn, &person_a).await?;
        persons_repository::create_person(&mut conn, &person_b).await?;
        update_candidate_list(&mut conn, &list_id, &[person_a.id, person_b.id]).await?;

        let detail = get_candidate_list(&mut conn, &list_id)
            .await?
            .expect("candidate list");
        assert_eq!(detail.candidates.len(), 2);
        assert_eq!(detail.candidates[0].person.id, person_a.id);
        assert_eq!(detail.candidates[1].person.id, person_b.id);

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_list_returns_row_not_found(pool: PgPool) -> Result<(), sqlx::Error> {
        let mut conn = pool.acquire().await?;
        let err = update_candidate_list(&mut conn, &Uuid::new_v4(), &[])
            .await
            .unwrap_err();
        assert!(matches!(err, sqlx::Error::RowNotFound));

        Ok(())
    }
}
