use crate::core::{AnyLocale, ElectionType, ModelLocale as Locale};

pub fn generate_election_title(election_type: &ElectionType, locale: Locale) -> String {
    match (election_type, locale) {
        (ElectionType::Tk, Locale::Nl) => "de Tweede Kamer der Staten-Generaal".to_string(),
        (ElectionType::Tk, Locale::Fry) => "de Twadde Keamer fan de Steaten-Generaal".to_string(),

        (ElectionType::Ek, Locale::Nl) => "de Eerste Kamer der Staten-Generaal".to_string(),
        (ElectionType::Ek, Locale::Fry) => "de Earste Keamer fan de Steaten-Generaal".to_string(),

        (ElectionType::Ps(province), Locale::Nl) => {
            format!(
                "de provinciale staten van {}",
                province.title(AnyLocale::from(locale))
            )
        }
        (ElectionType::Ps(province), Locale::Fry) => {
            format!(
                "de Provinsjale Steaten fan {}",
                province.title(AnyLocale::from(locale))
            )
        }

        (ElectionType::Ws(council), Locale::Nl) => format!(
            "het algemeen bestuur van het waterschap {}",
            council.title(AnyLocale::from(locale))
        ),
        (ElectionType::Ws(council), Locale::Fry) => format!(
            "it algemien bestjoer fan it wetterskip {}",
            council.title(AnyLocale::from(locale))
        ),

        (ElectionType::Ep, Locale::Nl) => "het Europees Parlement".to_string(),
        (ElectionType::Ep, Locale::Fry) => "het Europees Parlement".to_string(),

        (ElectionType::Gr, _) => todo!("Support municipality regions"),
        (ElectionType::Kc, _) => todo!("Support electoral college regions"),
        (ElectionType::Er, _) => todo!("Support island regions"),
    }
}

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    use crate::core::election::{Province, WaterCouncil};

    use super::*;

    #[test]
    fn election_title() {
        // This test is not exhaustive
        assert_eq!(
            generate_election_title(&ElectionType::Ek, Locale::Nl),
            "de Eerste Kamer der Staten-Generaal"
        );
        assert_eq!(
            generate_election_title(&ElectionType::Tk, Locale::Fry),
            "de Twadde Keamer fan de Steaten-Generaal"
        );
    }

    #[test]
    fn election_tile_with_sub_type() {
        assert_eq!(
            generate_election_title(&ElectionType::Ps(Province::DR), Locale::Nl),
            "de provinciale staten van Drenthe"
        );

        assert_eq!(
            generate_election_title(&ElectionType::Ws(WaterCouncil::Fryslan), Locale::Fry),
            "it algemien bestjoer fan it wetterskip Fryslân"
        );
    }

    #[test]
    fn unimplemented() {
        assert!(catch_unwind(|| generate_election_title(&ElectionType::Gr, Locale::Nl)).is_err());

        assert!(catch_unwind(|| generate_election_title(&ElectionType::Kc, Locale::Fry)).is_err());

        assert!(catch_unwind(|| generate_election_title(&ElectionType::Er, Locale::Nl)).is_err());
    }
}
