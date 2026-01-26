use crate::political_groups::{
    AuthorisedAgentId, ListSubmitter, ListSubmitterId, PoliticalGroup, PoliticalGroupId,
};
use chrono::Utc;
use sqlx::PgConnection;

pub async fn get_political_groups(
    conn: &mut PgConnection,
) -> Result<Vec<PoliticalGroup>, sqlx::Error> {
    sqlx::query_as!(
        PoliticalGroup,
        r#"
        SELECT id,
               authorised_agent_id AS "authorised_agent_id:AuthorisedAgentId",
               list_submitter_id AS "list_submitter_id:ListSubmitterId",
               legal_name,
               display_name,
               created_at,
               updated_at
        FROM political_groups
        ORDER BY display_name
        "#,
    )
    .fetch_all(conn)
    .await
}

pub async fn get_political_group(
    conn: &mut PgConnection,
    id: &mut PoliticalGroupId,
) -> Result<PoliticalGroup, sqlx::Error> {
    sqlx::query_as!(
        PoliticalGroup,
        r#"
        SELECT id,
               authorised_agent_id AS "authorised_agent_id:AuthorisedAgentId",
               list_submitter_id AS "list_submitter_id:ListSubmitterId",
               legal_name,
               display_name,
               created_at,
               updated_at
        FROM political_groups
        WHERE id = $1
        "#,
        id.uuid()
    )
    .fetch_one(conn)
    .await
}

pub async fn create_political_group(
    conn: &mut PgConnection,
    political_group: &PoliticalGroup,
) -> Result<PoliticalGroup, sqlx::Error> {
    sqlx::query_as!(
        PoliticalGroup,
        r#"
        INSERT INTO political_groups (id, authorised_agent_id, list_submitter_id, legal_name, display_name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id,
            authorised_agent_id AS "authorised_agent_id:AuthorisedAgentId",
            list_submitter_id AS "list_submitter_id:ListSubmitterId",
            legal_name,
            display_name,
            created_at,
            updated_at
        "#,
        political_group.id.uuid(),
        &political_group.authorised_agent_id as _,
        &political_group.list_submitter_id as _,
        &political_group.legal_name,
        &political_group.display_name,
        &political_group.created_at,
        &political_group.updated_at
    ).fetch_one(conn).await
}

pub async fn get_list_submitters(
    conn: &mut PgConnection,
    political_group_id: &PoliticalGroupId,
) -> Result<Vec<ListSubmitter>, sqlx::Error> {
    sqlx::query_as!(
        ListSubmitter,
        r#"
        SELECT id,
               last_name,
               last_name_prefix,
               initials,
               locality,
               postal_code,
               house_number,
               house_number_addition,
               street_name,
               created_at,
               updated_at
        FROM list_submitters
        WHERE political_group_id = $1
        "#,
        political_group_id.uuid()
    )
    .fetch_all(conn)
    .await
}

pub async fn get_list_submitter(
    conn: &mut PgConnection,
    political_group_id: &PoliticalGroupId,
    submitter_id: &mut ListSubmitterId,
) -> Result<ListSubmitter, sqlx::Error> {
    sqlx::query_as!(
        ListSubmitter,
        r#"
        SELECT id,
               last_name,
               last_name_prefix,
               initials,
               locality,
               postal_code,
               house_number,
               house_number_addition,
               street_name,
               created_at,
               updated_at
        FROM list_submitters
        WHERE political_group_id = $1 
          AND id = $2
        "#,
        political_group_id.uuid(),
        submitter_id.uuid()
    )
    .fetch_one(conn)
    .await
}

pub async fn create_list_submitter(
    conn: &mut PgConnection,
    political_group_id: &PoliticalGroupId,
    list_submitter: &ListSubmitter,
) -> Result<ListSubmitter, sqlx::Error> {
    sqlx::query_as!(
        ListSubmitter,
        r#"
        INSERT INTO list_submitters (id,
                                     political_group_id,
                                     last_name,
                                     last_name_prefix,
                                     initials,
                                     locality,
                                     postal_code,
                                     house_number,
                                     house_number_addition,
                                     street_name,
                                     created_at,
                                     updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING
            id,
            last_name,
            last_name_prefix,
            initials,
            locality,
            postal_code,
            house_number,
            house_number_addition,
            street_name,
            created_at,
            updated_at
        "#,
        list_submitter.id.uuid(),
        political_group_id.uuid(),
        list_submitter.last_name,
        list_submitter.last_name_prefix,
        list_submitter.initials,
        list_submitter.locality,
        list_submitter.postal_code,
        list_submitter.house_number,
        list_submitter.house_number_addition,
        list_submitter.street_name,
        Utc::now(),
        Utc::now(),
    )
    .fetch_one(conn)
    .await
}
