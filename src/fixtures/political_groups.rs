use uuid::Uuid;

use crate::{
    AppError, AppStore, DisplayName, DutchAddress, FullName, HouseNumber, HouseNumberAddition,
    Initials, LastName, LastNamePrefix, LegalName, Locality, PostalCode, StreetName, UtcDateTime,
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
        legal_name: Some(
            "Kiesraad Demo Partij"
                .parse::<LegalName>()
                .expect("legal name"),
        ),
        display_name: Some(
            "Kiesraad Demo"
                .parse::<DisplayName>()
                .expect("display name"),
        ),
        created_at: UtcDateTime::now(),
        updated_at: UtcDateTime::now(),
    };

    political_group.update(store).await?;

    AuthorisedAgent {
        id: agent_id,
        name: FullName {
            last_name: "Jansen".parse::<LastName>().expect("last name"),
            last_name_prefix: Some("de".parse::<LastNamePrefix>().expect("last name prefix")),
            initials: "A.B.".parse::<Initials>().expect("initials"),
        },
        created_at: UtcDateTime::now(),
        updated_at: UtcDateTime::now(),
    }
    .create(store)
    .await?;

    ListSubmitter {
        id: submitter_id,
        name: FullName {
            last_name: "Bos".parse::<LastName>().expect("last name"),
            last_name_prefix: None,
            initials: "E.F.".parse::<Initials>().expect("initials"),
        },
        address: DutchAddress {
            locality: Some("Rotterdam".parse::<Locality>().expect("locality")),
            postal_code: Some("3011 CC".parse::<PostalCode>().expect("postal code")),
            house_number: Some("5".parse::<HouseNumber>().expect("house number")),
            house_number_addition: Some(
                "B".parse::<HouseNumberAddition>()
                    .expect("house number addition"),
            ),
            street_name: Some("Coolsingel".parse::<StreetName>().expect("street name")),
        },
        created_at: UtcDateTime::now(),
        updated_at: UtcDateTime::now(),
    }
    .create(store)
    .await?;

    SubstituteSubmitter {
        id: substitute_submitter_id_1,
        name: FullName {
            last_name: "Smit".parse::<LastName>().expect("last name"),
            last_name_prefix: Some("van".parse::<LastNamePrefix>().expect("last name prefix")),
            initials: "G.H.".parse::<Initials>().expect("initials"),
        },
        address: DutchAddress {
            locality: Some("Den Haag".parse::<Locality>().expect("locality")),
            postal_code: Some("2511 DD".parse::<PostalCode>().expect("postal code")),
            house_number: Some("18".parse::<HouseNumber>().expect("house number")),
            house_number_addition: None,
            street_name: Some("Spui".parse::<StreetName>().expect("street name")),
        },
        created_at: UtcDateTime::now(),
        updated_at: UtcDateTime::now(),
    }
    .create(store)
    .await?;

    SubstituteSubmitter {
        id: substitute_submitter_id_2,
        name: FullName {
            last_name: "Jong".parse::<LastName>().expect("last name"),
            last_name_prefix: None,
            initials: "I.J.".parse::<Initials>().expect("initials"),
        },
        address: DutchAddress {
            locality: Some("Utrecht".parse::<Locality>().expect("locality")),
            postal_code: Some("3511 AA".parse::<PostalCode>().expect("postal code")),
            house_number: Some("21".parse::<HouseNumber>().expect("house number")),
            house_number_addition: Some(
                "C".parse::<HouseNumberAddition>()
                    .expect("house number addition"),
            ),
            street_name: Some("Oudegracht".parse::<StreetName>().expect("street name")),
        },
        created_at: UtcDateTime::now(),
        updated_at: UtcDateTime::now(),
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
        assert_eq!(list_submitters.len(), 1);

        let substitute_submitters = store.get_substitute_submitters().unwrap();
        assert_eq!(substitute_submitters.len(), 2);
    }
}
