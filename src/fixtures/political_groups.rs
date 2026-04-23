use crate::{
    AppError, AppStore,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    common::{
        Address, DisplayName, DutchAddress, FullName, HouseNumber, HouseNumberAddition, Initials,
        LastName, LastNamePrefix, LegalName, Locality, PostalCode, StreetName,
    },
    list_submitters::{ListSubmitter, ListSubmitterId},
    political_groups::PoliticalGroup,
};
use uuid::Uuid;

pub async fn load(store: &AppStore) -> Result<(), AppError> {
    let agent_id: AuthorisedAgentId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_authorised_agent").into();

    let submitter_id: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_list_submitter").into();

    let substitute_submitter_id_1: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_substitute_submitter_1").into();
    let substitute_submitter_id_2: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_substitute_submitter_2").into();

    let political_group = PoliticalGroup {
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
    };

    political_group.update(store).await?;

    AuthorisedAgent {
        id: agent_id,
        name: FullName {
            first_name: None,
            last_name: "Jansen".parse::<LastName>().expect("last name"),
            last_name_prefix: Some("de".parse::<LastNamePrefix>().expect("last name prefix")),
            initials: "A.B.".parse::<Initials>().expect("initials"),
        },
    }
    .create(store)
    .await?;

    ListSubmitter {
        id: submitter_id,
        name: FullName {
            first_name: None,
            last_name: "Bos".parse::<LastName>().expect("last name"),
            last_name_prefix: None,
            initials: "E.F.".parse::<Initials>().expect("initials"),
        },
        address: Address::Dutch(DutchAddress {
            locality: Some("Rotterdam".parse::<Locality>().expect("locality")),
            postal_code: Some("3011 CC".parse::<PostalCode>().expect("postal code")),
            house_number: Some("5".parse::<HouseNumber>().expect("house number")),
            house_number_addition: Some(
                "B".parse::<HouseNumberAddition>()
                    .expect("house number addition"),
            ),
            street_name: Some("Coolsingel".parse::<StreetName>().expect("street name")),
        }),
    }
    .update(store)
    .await?;

    ListSubmitter {
        id: substitute_submitter_id_1,
        name: FullName {
            first_name: None,
            last_name: "Smit".parse::<LastName>().expect("last name"),
            last_name_prefix: Some("van".parse::<LastNamePrefix>().expect("last name prefix")),
            initials: "G.H.".parse::<Initials>().expect("initials"),
        },
        address: Address::Dutch(DutchAddress {
            locality: Some("Den Haag".parse::<Locality>().expect("locality")),
            postal_code: Some("2511 DD".parse::<PostalCode>().expect("postal code")),
            house_number: Some("18".parse::<HouseNumber>().expect("house number")),
            house_number_addition: None,
            street_name: Some("Spui".parse::<StreetName>().expect("street name")),
        }),
    }
    .create_substitute(store)
    .await?;

    ListSubmitter {
        id: substitute_submitter_id_2,
        name: FullName {
            first_name: None,
            last_name: "Jong".parse::<LastName>().expect("last name"),
            last_name_prefix: None,
            initials: "I.J.".parse::<Initials>().expect("initials"),
        },
        address: Address::Dutch(DutchAddress {
            locality: Some("Utrecht".parse::<Locality>().expect("locality")),
            postal_code: Some("3511 AA".parse::<PostalCode>().expect("postal code")),
            house_number: Some("21".parse::<HouseNumber>().expect("house number")),
            house_number_addition: Some(
                "C".parse::<HouseNumberAddition>()
                    .expect("house number addition"),
            ),
            street_name: Some("Oudegracht".parse::<StreetName>().expect("street name")),
        }),
    }
    .create_substitute(store)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load() {
        let store = AppStore::new_for_test();
        load(&store).await.unwrap();

        let list_submitter = store.get_list_submitter();
        assert!(list_submitter.is_complete());

        let substitute_submitters = store.get_substitute_submitters();
        assert_eq!(substitute_submitters.len(), 2);
    }
}
