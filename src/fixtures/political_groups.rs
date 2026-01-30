use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AppError,
    political_groups::{
        self, AuthorisedAgent, AuthorisedAgentId, ListSubmitter, ListSubmitterId, PoliticalGroup,
        PoliticalGroupId,
    },
};

pub async fn load(db: &PgPool) -> Result<(), AppError> {
    let political_group_id: PoliticalGroupId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_political_group").into();

    let agent_1_id: AuthorisedAgentId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_authorised_agent_1").into();
    let agent_2_id: AuthorisedAgentId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_authorised_agent_2").into();

    let submitter_1_id: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_list_submitter_1").into();
    let submitter_2_id: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_list_submitter_2").into();

    let political_group = PoliticalGroup {
        id: political_group_id,
        long_list_allowed: None,
        legal_name: "Kiesraad Demo Partij".to_string(),
        legal_name_confirmed: None,
        display_name: "Kiesraad Demo".to_string(),
        display_name_confirmed: None,
        authorised_agent_id: None,
        list_submitter_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let political_group = political_groups::create_political_group(db, &political_group).await?;

    political_groups::create_authorised_agent(
        db,
        political_group.id,
        &AuthorisedAgent {
            id: agent_1_id,
            last_name: "Jansen".to_string(),
            last_name_prefix: Some("de".to_string()),
            initials: "A.B.".to_string(),
            locality: Some("Utrecht".to_string()),
            postal_code: Some("3511 AA".to_string()),
            house_number: Some("10".to_string()),
            house_number_addition: Some("A".to_string()),
            street_name: Some("Oude Gracht".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    )
    .await?;

    political_groups::create_authorised_agent(
        db,
        political_group.id,
        &AuthorisedAgent {
            id: agent_2_id,
            last_name: "Visser".to_string(),
            last_name_prefix: None,
            initials: "C.D.".to_string(),
            locality: Some("Amersfoort".to_string()),
            postal_code: Some("3811 BB".to_string()),
            house_number: Some("25".to_string()),
            house_number_addition: None,
            street_name: Some("Langegracht".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    )
    .await?;

    political_groups::create_list_submitter(
        db,
        political_group.id,
        &ListSubmitter {
            id: submitter_1_id,
            last_name: "Bos".to_string(),
            last_name_prefix: None,
            initials: "E.F.".to_string(),
            locality: Some("Rotterdam".to_string()),
            postal_code: Some("3011 CC".to_string()),
            house_number: Some("5".to_string()),
            house_number_addition: Some("B".to_string()),
            street_name: Some("Coolsingel".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    )
    .await?;

    political_groups::create_list_submitter(
        db,
        political_group.id,
        &ListSubmitter {
            id: submitter_2_id,
            last_name: "Smit".to_string(),
            last_name_prefix: Some("van".to_string()),
            initials: "G.H.".to_string(),
            locality: Some("Den Haag".to_string()),
            postal_code: Some("2511 DD".to_string()),
            house_number: Some("18".to_string()),
            house_number_addition: None,
            street_name: Some("Spui".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::{PgConnection, PgPool};

    use super::*;

    pub async fn get_political_groups(
        conn: &mut PgConnection,
    ) -> Result<Vec<PoliticalGroup>, sqlx::Error> {
        sqlx::query_as!(
            PoliticalGroup,
            r#"
            SELECT id,
                long_list_allowed,
                legal_name,
                legal_name_confirmed,
                display_name,
                display_name_confirmed,
                authorised_agent_id AS "authorised_agent_id:AuthorisedAgentId",
                list_submitter_id AS "list_submitter_id:ListSubmitterId",
                created_at,
                updated_at
            FROM political_groups
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(conn)
        .await
    }

    #[sqlx::test]
    async fn test_load(pool: PgPool) {
        let mut conn = pool.acquire().await.unwrap();
        load(&pool).await.unwrap();

        let groups = get_political_groups(&mut conn).await.unwrap();
        assert_eq!(groups.len(), 1);

        let list_submitters = political_groups::get_list_submitters(&pool, groups[0].id)
            .await
            .unwrap();
        assert_eq!(list_submitters.len(), 2);

        let authorised_count = political_groups::get_authorised_agents(&pool, groups[0].id)
            .await
            .unwrap()
            .len();
        assert_eq!(authorised_count, 2);
    }
}
