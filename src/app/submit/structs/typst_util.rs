use crate::core::{AnyLocale, ElectionConfig, ElectionType, ModelLocale as Locale};

pub fn generate_election_title(election: &ElectionConfig, locale: Locale) -> String {
    let region = || {
        election
            .region_title(AnyLocale::from(locale))
            .expect("region title required for this election type")
    };

    match (election.election_type(), locale) {
        (ElectionType::Tk, Locale::Nl) => "de Tweede Kamer der Staten-Generaal".to_string(),
        (ElectionType::Tk, Locale::Fry) => "de Twadde Keamer fan de Steaten-Generaal".to_string(),

        (ElectionType::Ek, Locale::Nl) => "de Eerste Kamer der Staten-Generaal".to_string(),
        (ElectionType::Ek, Locale::Fry) => "de Earste Keamer fan de Steaten-Generaal".to_string(),

        (ElectionType::Ps, Locale::Nl) => format!("de provinciale staten van {}", region()),
        (ElectionType::Ps, Locale::Fry) => format!("de Provinsjale Steaten fan {}", region()),

        (ElectionType::Ws, Locale::Nl) => {
            format!("het algemeen bestuur van het waterschap {}", region())
        }
        (ElectionType::Ws, Locale::Fry) => {
            format!("it algemien bestjoer fan it wetterskip {}", region())
        }

        (ElectionType::Ep, Locale::Nl) => "het Europees Parlement".to_string(),
        (ElectionType::Ep, Locale::Fry) => "het Europees Parlement".to_string(),

        (ElectionType::Gr, _) => todo!("Support municipality regions"),
        (ElectionType::Kc, _) => todo!("Support electoral college regions"),
        (ElectionType::Er, _) => todo!("Support island regions"),
    }
}

#[cfg(test)]
mod tests {
    use crate::core::election::{Province, WaterCouncil};

    use super::*;

    #[test]
    fn election_title() {
        assert_eq!(
            generate_election_title(&ElectionConfig::EK27, Locale::Nl),
            "de Eerste Kamer der Staten-Generaal"
        );
        assert_eq!(
            generate_election_title(&ElectionConfig::EK27, Locale::Fry),
            "de Earste Keamer fan de Steaten-Generaal"
        );
    }

    #[test]
    fn election_tile_with_sub_type() {
        assert_eq!(
            generate_election_title(&ElectionConfig::PS27(Province::DR), Locale::Nl),
            "de provinciale staten van Drenthe"
        );

        assert_eq!(
            generate_election_title(&ElectionConfig::WS27(WaterCouncil::Fryslan), Locale::Fry),
            "it algemien bestjoer fan it wetterskip Fryslân"
        );
    }
}
