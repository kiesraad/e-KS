use crate::political_groups::{
    AuthorisedAgent, AuthorisedAgentId, ListSubmitter, ListSubmitterId, PoliticalGroup,
    PoliticalGroupId,
};
use chrono::Utc;
use sqlx::PgPool;

pub async fn get_single_political_group(
    db: &PgPool,
) -> Result<Option<PoliticalGroup>, sqlx::Error> {
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
        LIMIT 1
        "#,
    )
    .fetch_optional(db)
    .await
}

pub async fn update_political_group(
    db: &PgPool,
    political_group: &PoliticalGroup,
) -> Result<PoliticalGroup, sqlx::Error> {
    sqlx::query_as!(
        PoliticalGroup,
        r#"
        UPDATE political_groups
        SET
            long_list_allowed = $1,
            legal_name = $2,
            legal_name_confirmed = $3,
            display_name = $4,
            display_name_confirmed = $5,
            authorised_agent_id = $6,
            list_submitter_id = $7,
            updated_at = $8
        WHERE id = $9
        RETURNING id,
            long_list_allowed,
            legal_name,
            legal_name_confirmed,
            display_name,
            display_name_confirmed,
            authorised_agent_id AS "authorised_agent_id:AuthorisedAgentId",
            list_submitter_id AS "list_submitter_id:ListSubmitterId",
            created_at,
            updated_at
        "#,
        political_group.long_list_allowed,
        &political_group.legal_name,
        political_group.legal_name_confirmed,
        &political_group.display_name,
        political_group.display_name_confirmed,
        &political_group.authorised_agent_id as _,
        &political_group.list_submitter_id as _,
        Utc::now(),
        political_group.id.uuid(),
    )
    .fetch_one(db)
    .await
}

#[cfg(any(test, feature = "fixtures"))]
pub async fn create_political_group(
    db: &PgPool,
    political_group: &PoliticalGroup,
) -> Result<PoliticalGroup, sqlx::Error> {
    sqlx::query_as!(
        PoliticalGroup,
        r#"
        INSERT INTO political_groups (
            id,
            long_list_allowed,
            legal_name,
            legal_name_confirmed,
            display_name,
            display_name_confirmed,
            authorised_agent_id,
            list_submitter_id,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id,
            long_list_allowed,
            legal_name,
            legal_name_confirmed,
            display_name,
            display_name_confirmed,
            authorised_agent_id AS "authorised_agent_id:AuthorisedAgentId",
            list_submitter_id AS "list_submitter_id:ListSubmitterId",
            created_at,
            updated_at
        "#,
        political_group.id.uuid(),
        political_group.long_list_allowed,
        &political_group.legal_name,
        political_group.legal_name_confirmed,
        &political_group.display_name,
        political_group.display_name_confirmed,
        &political_group.authorised_agent_id as _,
        &political_group.list_submitter_id as _,
        &political_group.created_at,
        &political_group.updated_at
    )
    .fetch_one(db)
    .await
}

pub async fn get_list_submitters(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
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
    .fetch_all(db)
    .await
}

pub async fn get_authorised_agents(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
) -> Result<Vec<AuthorisedAgent>, sqlx::Error> {
    sqlx::query_as!(
        AuthorisedAgent,
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
        FROM authorised_agents
        WHERE political_group_id = $1
        "#,
        political_group_id.uuid()
    )
    .fetch_all(db)
    .await
}

#[cfg(any(test, feature = "fixtures"))]
pub async fn create_authorised_agent(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    authorised_agent: &AuthorisedAgent,
) -> Result<AuthorisedAgent, sqlx::Error> {
    sqlx::query_as!(
        AuthorisedAgent,
        r#"
        INSERT INTO authorised_agents (id,
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
        authorised_agent.id.uuid(),
        political_group_id.uuid(),
        authorised_agent.last_name,
        authorised_agent.last_name_prefix,
        authorised_agent.initials,
        authorised_agent.locality,
        authorised_agent.postal_code,
        authorised_agent.house_number,
        authorised_agent.house_number_addition,
        authorised_agent.street_name,
        Utc::now(),
        Utc::now(),
    )
    .fetch_one(db)
    .await
}

pub async fn get_list_submitter(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    submitter_id: &ListSubmitterId,
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
    .fetch_one(db)
    .await
}

pub async fn set_default_list_submitter(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    submitter_id: Option<ListSubmitterId>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE political_groups
        SET list_submitter_id = $1
        WHERE id = $2
        "#,
        submitter_id.map(|id| id.uuid()),
        political_group_id.uuid()
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn create_list_submitter(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
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
    .fetch_one(db)
    .await
}

pub async fn update_list_submitter(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    list_submitter: &ListSubmitter,
) -> Result<ListSubmitter, sqlx::Error> {
    sqlx::query_as!(
        ListSubmitter,
        r#"
        UPDATE list_submitters
        SET
            last_name = $1,
            last_name_prefix = $2,
            initials = $3,
            locality = $4,
            postal_code = $5,
            house_number = $6,
            house_number_addition = $7,
            street_name = $8,
            updated_at = $9
        WHERE political_group_id = $10
          AND id = $11
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
        list_submitter.last_name,
        list_submitter.last_name_prefix,
        list_submitter.initials,
        list_submitter.locality,
        list_submitter.postal_code,
        list_submitter.house_number,
        list_submitter.house_number_addition,
        list_submitter.street_name,
        Utc::now(),
        political_group_id.uuid(),
        list_submitter.id.uuid(),
    )
    .fetch_one(db)
    .await
}

pub async fn remove_list_submitter(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    list_submitter_id: ListSubmitterId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE political_groups
        SET list_submitter_id = NULL
        WHERE id = $1
          AND list_submitter_id = $2
        "#,
        political_group_id.uuid(),
        list_submitter_id.uuid()
    )
    .execute(db)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM list_submitters
        WHERE political_group_id = $1
          AND id = $2
        "#,
        political_group_id.uuid(),
        list_submitter_id.uuid()
    )
    .execute(db)
    .await?;

    Ok(())
}
