use crate::{
    AppError, PgStore,
    structs::{
        common::{Address, DisplayName, DutchAddress, FullName},
        list_designation::ListDesignation,
        list_submitters::{ListSubmitter, ListSubmitterId},
        name_authorisations::{NameAuthorisation, NameAuthorisationId},
        political_groups::PoliticalGroup,
    },
};
use uuid::Uuid;

pub async fn load(store: &PgStore, display_name: Option<DisplayName>) -> Result<(), AppError> {
    let agent_id: NameAuthorisationId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_authorised_agent").into();

    let submitter_id: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_list_submitter").into();

    let substitute_submitter_id_1: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_substitute_submitter_1").into();
    let substitute_submitter_id_2: ListSubmitterId =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, b"fixture_substitute_submitter_2").into();

    let political_group = PoliticalGroup {
        display_name: Some(
            display_name.unwrap_or_else(|| "Kiesraad Demo".parse().expect("display name")),
        ),
        list_designation: Some(ListDesignation::Standalone),
        previous_election_results: None,
    };

    political_group.update(store).await?;

    NameAuthorisation {
        id: agent_id,
        name: FullName {
            first_name: None,
            last_name: "Jansen".parse().expect("last name"),
            last_name_prefix: Some("de".parse().expect("last name prefix")),
            initials: "A.B.".parse().expect("initials"),
        },
        legal_name: "Kiesraad Demo Partij".parse().expect("legal name"),
    }
    .create(store)
    .await?;

    ListSubmitter {
        id: submitter_id,
        name: FullName {
            first_name: None,
            last_name: "Bos".parse().expect("last name"),
            last_name_prefix: None,
            initials: "E.F.".parse().expect("initials"),
        },
        address: Address::Dutch(DutchAddress {
            locality: Some("Rotterdam".parse().expect("locality")),
            postal_code: Some("3011 CC".parse().expect("postal code")),
            house_number: Some("5".parse().expect("house number")),
            house_number_addition: Some("B".parse().expect("house number addition")),
            street_name: Some("Coolsingel".parse().expect("street name")),
            known_in_bag: Some(true),
        }),
        is_substitute: false,
    }
    .update(store)
    .await?;

    ListSubmitter {
        id: substitute_submitter_id_1,
        name: FullName {
            first_name: None,
            last_name: "Smit".parse().expect("last name"),
            last_name_prefix: Some("van".parse().expect("last name prefix")),
            initials: "G.H.".parse().expect("initials"),
        },
        address: Address::Dutch(DutchAddress {
            locality: Some("Den Haag".parse().expect("locality")),
            postal_code: Some("2511 DD".parse().expect("postal code")),
            house_number: Some("18".parse().expect("house number")),
            house_number_addition: None,
            street_name: Some("Spui".parse().expect("street name")),
            known_in_bag: Some(true),
        }),
        is_substitute: true,
    }
    .create_substitute(store)
    .await?;

    ListSubmitter {
        id: substitute_submitter_id_2,
        name: FullName {
            first_name: None,
            last_name: "Jong".parse().expect("last name"),
            last_name_prefix: None,
            initials: "I.J.".parse().expect("initials"),
        },
        address: Address::Dutch(DutchAddress {
            locality: Some("Utrecht".parse().expect("locality")),
            postal_code: Some("3511 AA".parse().expect("postal code")),
            house_number: Some("21".parse().expect("house number")),
            house_number_addition: Some("C".parse().expect("house number addition")),
            street_name: Some("Oudegracht".parse().expect("street name")),
            known_in_bag: Some(true),
        }),
        is_substitute: true,
    }
    .create_substitute(store)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::common::{HasSeverity, Problematic};

    #[tokio::test]
    async fn test_load() {
        let store = PgStore::new_for_test();
        load(&store, None).await.unwrap();

        let list_submitter = store.get_list_submitter();
        assert!(list_submitter.get_problems(()).is_all_good());

        let substitute_submitters = store.get_substitute_submitters();
        assert_eq!(substitute_submitters.len(), 2);
    }
}
