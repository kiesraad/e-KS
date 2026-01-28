use chrono::Utc;
use sqlx::PgPool;

use crate::{
    candidate_lists::{self, Candidate, CandidateListId, FullCandidateList},
    persons::{Gender, Person, PersonId},
};

pub async fn get_full_candidate_list(
    db: &PgPool,
    list_id: CandidateListId,
) -> Result<Option<FullCandidateList>, sqlx::Error> {
    let list = candidate_lists::get_candidate_list(db, list_id).await?;

    let Some(list) = list else {
        return Ok(None);
    };

    let candidates = sqlx::query_file!(
        "sql/candidate_lists/list_candidates_for_list.sql",
        list.id.uuid()
    )
    .fetch_all(db)
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
            place_of_residence: row.place_of_residence,
            country_of_residence: row.country_of_residence,
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

    Ok(Some(FullCandidateList { list, candidates }))
}

pub async fn update_candidate_list_order(
    db: &PgPool,
    list_id: CandidateListId,
    person_ids: &[PersonId],
) -> Result<FullCandidateList, sqlx::Error> {
    let mut tx = db.begin().await?;

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

    let positions: Vec<i32> = (1..=person_ids.len() as i32).collect();

    sqlx::query!(
        r#"
        INSERT INTO candidate_lists_persons (candidate_list_id, person_id, position, created_at, updated_at)
        SELECT $1, person_id, position, $4, $5
        FROM UNNEST($2::uuid[], $3::int[]) AS t(person_id, position)
        "#,
        list_id.uuid(),
        &person_ids.iter().map(|p| p.uuid()).collect::<Vec<_>>(),
        &positions,
        Utc::now(),
        Utc::now()
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    get_full_candidate_list(db, list_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn append_candidate_to_list(
    db: &PgPool,
    list_id: CandidateListId,
    person_id: PersonId,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

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
        INSERT INTO candidate_lists_persons (candidate_list_id, person_id, position, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        list_id.uuid(),
        person_id.uuid(),
        new_position,
        Utc::now(),
        Utc::now()
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn remove_candidate(db: &PgPool, person_id: PersonId) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query!(
        r#"
        UPDATE candidate_lists
        SET updated_at = $2
        FROM candidate_lists_persons
        WHERE candidate_lists.id = candidate_lists_persons.candidate_list_id
        AND candidate_lists_persons.person_id = $1
        "#,
        person_id.uuid(),
        Utc::now()
    )
    .execute(&mut *tx)
    .await?;

    let deleted = sqlx::query!(
        r#"
        DELETE FROM candidate_lists_persons
        WHERE person_id = $1
        "#,
        person_id.uuid(),
    )
    .execute(&mut *tx)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    tx.commit().await?;

    Ok(())
}

pub async fn remove_candidate_from_list(
    db: &PgPool,
    list_id: CandidateListId,
    person_id: PersonId,
) -> Result<(), sqlx::Error> {
    let updated = sqlx::query!(
        r#"
        UPDATE candidate_lists
        SET updated_at = NOW()
        WHERE id = $1
        "#,
        list_id.uuid(),
    )
    .execute(db)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    let deleted = sqlx::query!(
        r#"
        DELETE FROM candidate_lists_persons
        WHERE candidate_list_id = $1 AND person_id = $2
        "#,
        list_id.uuid(),
        person_id.uuid(),
    )
    .execute(db)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn get_candidate(
    db: &PgPool,
    list_id: CandidateListId,
    person_id: PersonId,
) -> Result<Candidate, sqlx::Error> {
    let person = crate::persons::get_person(db, person_id)
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
    .fetch_one(db)
    .await?;

    Ok(Candidate {
        list_id,
        position: record.position as usize,
        person,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    use crate::{
        candidate_lists, persons,
        test_utils::{sample_candidate_list, sample_person_with_last_name},
    };

    #[sqlx::test]
    async fn get_candidate_list_includes_candidates(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person_a).await?;
        persons::create_person(&pool, &person_b).await?;
        update_candidate_list_order(&pool, list_id, &[person_a.id, person_b.id]).await?;

        let detail = get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(2, detail.candidates.len());
        assert_eq!(person_a.id, detail.candidates[0].person.id);
        assert_eq!(person_b.id, detail.candidates[1].person.id);

        Ok(())
    }

    #[sqlx::test]
    async fn update_candidate_list_order_returns_row_not_found(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let err = update_candidate_list_order(&pool, CandidateListId::new(), &[])
            .await
            .unwrap_err();
        assert!(matches!(err, sqlx::Error::RowNotFound));

        Ok(())
    }

    #[sqlx::test]
    async fn get_full_candidate_list_returns_none_for_missing_list(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let missing = get_full_candidate_list(&pool, CandidateListId::new()).await?;
        assert!(missing.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_append_candidate_to_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person_a).await?;
        persons::create_person(&pool, &person_b).await?;

        append_candidate_to_list(&pool, list_id, person_a.id).await?;
        append_candidate_to_list(&pool, list_id, person_b.id).await?;

        let detail = get_full_candidate_list(&pool, list_id)
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
    async fn append_candidate_to_list_returns_row_not_found(
        pool: PgPool,
    ) -> Result<(), sqlx::Error> {
        let err = append_candidate_to_list(&pool, CandidateListId::new(), PersonId::new())
            .await
            .unwrap_err();
        assert!(matches!(err, sqlx::Error::RowNotFound));

        Ok(())
    }

    #[sqlx::test]
    async fn remove_candidate_removes_from_list(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person_a = sample_person_with_last_name(PersonId::new(), "Jansen");
        let person_b = sample_person_with_last_name(PersonId::new(), "Bakker");

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person_a).await?;
        persons::create_person(&pool, &person_b).await?;
        append_candidate_to_list(&pool, list_id, person_a.id).await?;
        append_candidate_to_list(&pool, list_id, person_b.id).await?;

        remove_candidate(&pool, person_a.id).await?;

        let detail = get_full_candidate_list(&pool, list_id)
            .await?
            .expect("candidate list");
        assert_eq!(detail.candidates.len(), 1);
        assert_eq!(detail.candidates[0].person.id, person_b.id);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_candidate_returns_row_not_found(pool: PgPool) -> Result<(), sqlx::Error> {
        let err = remove_candidate(&pool, PersonId::new()).await.unwrap_err();
        assert!(matches!(err, sqlx::Error::RowNotFound));

        Ok(())
    }

    #[sqlx::test]
    async fn get_candidate_returns_candidate(pool: PgPool) -> Result<(), sqlx::Error> {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person_with_last_name(PersonId::new(), "Jansen");

        candidate_lists::create_candidate_list(&pool, &list).await?;
        persons::create_person(&pool, &person).await?;
        append_candidate_to_list(&pool, list_id, person.id).await?;

        let candidate = get_candidate(&pool, list_id, person.id).await?;
        assert_eq!(candidate.list_id, list_id);
        assert_eq!(candidate.position, 1);
        assert_eq!(candidate.person.id, person.id);

        Ok(())
    }
}
