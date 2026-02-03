use crate::political_groups::{
    AuthorisedAgent, AuthorisedAgentId, ListSubmitter, ListSubmitterId, PoliticalGroup,
    PoliticalGroupId, SubstituteSubmitter, SubstituteSubmitterId,
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
               display_name,
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
            display_name = $3,
            updated_at = $4
        WHERE id = $5
        RETURNING id,
            long_list_allowed,
            legal_name,
            display_name,
            created_at,
            updated_at
        "#,
        political_group.long_list_allowed,
        political_group.legal_name,
        political_group.display_name,
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
            display_name,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id,
            long_list_allowed,
            legal_name,
            display_name,
            created_at,
            updated_at
        "#,
        political_group.id.uuid(),
        political_group.long_list_allowed,
        political_group.legal_name,
        political_group.display_name,
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

pub async fn get_substitute_submitters(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
) -> Result<Vec<SubstituteSubmitter>, sqlx::Error> {
    sqlx::query_as!(
        SubstituteSubmitter,
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
        FROM substitute_submitters
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
                                      created_at,
                                      updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id,
            last_name,
            last_name_prefix,
            initials,
            created_at,
            updated_at
        "#,
        authorised_agent.id.uuid(),
        political_group_id.uuid(),
        authorised_agent.last_name,
        authorised_agent.last_name_prefix,
        authorised_agent.initials,
        Utc::now(),
        Utc::now(),
    )
    .fetch_one(db)
    .await
}

pub async fn get_authorised_agent(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    agent_id: &AuthorisedAgentId,
) -> Result<AuthorisedAgent, sqlx::Error> {
    sqlx::query_as!(
        AuthorisedAgent,
        r#"
        SELECT id,
               last_name,
               last_name_prefix,
               initials,
               created_at,
               updated_at
        FROM authorised_agents
        WHERE political_group_id = $1
          AND id = $2
        "#,
        political_group_id.uuid(),
        agent_id.uuid()
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

pub async fn get_substitute_submitter(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    submitter_id: &SubstituteSubmitterId,
) -> Result<SubstituteSubmitter, sqlx::Error> {
    sqlx::query_as!(
        SubstituteSubmitter,
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
        FROM substitute_submitters
        WHERE political_group_id = $1
          AND id = $2
        "#,
        political_group_id.uuid(),
        submitter_id.uuid()
    )
    .fetch_one(db)
    .await
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

pub async fn create_substitute_submitter(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    substitute_submitter: &SubstituteSubmitter,
) -> Result<SubstituteSubmitter, sqlx::Error> {
    sqlx::query_as!(
        SubstituteSubmitter,
        r#"
        INSERT INTO substitute_submitters (id,
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
        substitute_submitter.id.uuid(),
        political_group_id.uuid(),
        substitute_submitter.last_name,
        substitute_submitter.last_name_prefix,
        substitute_submitter.initials,
        substitute_submitter.locality,
        substitute_submitter.postal_code,
        substitute_submitter.house_number,
        substitute_submitter.house_number_addition,
        substitute_submitter.street_name,
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

pub async fn update_substitute_submitter(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    substitute_submitter: &SubstituteSubmitter,
) -> Result<SubstituteSubmitter, sqlx::Error> {
    sqlx::query_as!(
        SubstituteSubmitter,
        r#"
        UPDATE substitute_submitters
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
        substitute_submitter.last_name,
        substitute_submitter.last_name_prefix,
        substitute_submitter.initials,
        substitute_submitter.locality,
        substitute_submitter.postal_code,
        substitute_submitter.house_number,
        substitute_submitter.house_number_addition,
        substitute_submitter.street_name,
        Utc::now(),
        political_group_id.uuid(),
        substitute_submitter.id.uuid(),
    )
    .fetch_one(db)
    .await
}

pub async fn update_authorised_agent(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    authorised_agent: &AuthorisedAgent,
) -> Result<AuthorisedAgent, sqlx::Error> {
    sqlx::query_as!(
        AuthorisedAgent,
        r#"
        UPDATE authorised_agents
        SET
            last_name = $1,
            last_name_prefix = $2,
            initials = $3,
            updated_at = $4
        WHERE political_group_id = $5
          AND id = $6
        RETURNING
            id,
            last_name,
            last_name_prefix,
            initials,
            created_at,
            updated_at
        "#,
        authorised_agent.last_name,
        authorised_agent.last_name_prefix,
        authorised_agent.initials,
        Utc::now(),
        political_group_id.uuid(),
        authorised_agent.id.uuid(),
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

pub async fn remove_substitute_submitter(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    substitute_submitter_id: SubstituteSubmitterId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM substitute_submitters
        WHERE political_group_id = $1
          AND id = $2
        "#,
        political_group_id.uuid(),
        substitute_submitter_id.uuid()
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn remove_authorised_agent(
    db: &PgPool,
    political_group_id: PoliticalGroupId,
    authorised_agent_id: AuthorisedAgentId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM authorised_agents
        WHERE political_group_id = $1
          AND id = $2
        "#,
        political_group_id.uuid(),
        authorised_agent_id.uuid()
    )
    .execute(db)
    .await?;

    Ok(())
}
