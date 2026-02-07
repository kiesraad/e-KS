use chrono::Utc;
use uuid::Uuid;

use crate::{
    AppError, AppStore,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    list_submitters::{ListSubmitter, ListSubmitterId},
    political_groups::{PoliticalGroup, PoliticalGroupId},
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

pub async fn load(store: &AppStore) -> Result<(), AppError> {
    let political_group_id: PoliticalGroupId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_political_group").into();

    let agent_id: AuthorisedAgentId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_authorised_agent").into();

    let submitter_id: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_list_submitter").into();

    let substitute_submitter_id_1: SubstituteSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_substitute_submitter_1").into();
    let substitute_submitter_id_2: SubstituteSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_substitute_submitter_2").into();

    let political_group = PoliticalGroup {
        id: political_group_id,
        long_list_allowed: None,
        legal_name: Some("Kiesraad Demo Partij".to_string()),
        display_name: Some("Kiesraad Demo".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    political_group.update(store).await?;

    AuthorisedAgent {
        id: agent_id,
        last_name: "Jansen".to_string(),
        last_name_prefix: Some("de".to_string()),
        initials: "A.B.".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
    .create(store)
    .await?;

    ListSubmitter {
        id: submitter_id,
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
    }
    .create(store)
    .await?;

    SubstituteSubmitter {
        id: substitute_submitter_id_1,
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
    }
    .create(store)
    .await?;

    SubstituteSubmitter {
        id: substitute_submitter_id_2,
        last_name: "De Jong".to_string(),
        last_name_prefix: None,
        initials: "I.J.".to_string(),
        locality: Some("Utrecht".to_string()),
        postal_code: Some("3511 AA".to_string()),
        house_number: Some("21".to_string()),
        house_number_addition: Some("C".to_string()),
        street_name: Some("Oudegracht".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
    .create(store)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_load(pool: PgPool) {
        let store = AppStore::new(pool);
        load(&store).await.unwrap();

        let list_submitters = store.get_list_submitters().unwrap();
        assert_eq!(list_submitters.len(), 2);
    }
}
