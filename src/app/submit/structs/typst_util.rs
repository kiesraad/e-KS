use crate::core::{AnyLocale, ElectionType, ModelLocale};

pub fn generate_election_title(election_type: &ElectionType, locale: ModelLocale) -> String {
    match (locale, election_type) {
        (ModelLocale::Nl, ElectionType::Tk) => "de Tweede Kamer der Staten-Generaal".to_string(),
        (ModelLocale::Nl, ElectionType::Ek) => "de Eerste Kamer der Staten-Generaal".to_string(),
        (ModelLocale::Nl, ElectionType::Ps(province)) => {
            format!(
                "de provinciale staten van {}",
                province.title(AnyLocale::from(locale))
            )
        }
        (ModelLocale::Nl, ElectionType::Ws(council)) => format!(
            "het algemeen bestuur van het waterschap {}",
            council.title(AnyLocale::from(locale))
        ),
        (ModelLocale::Nl, ElectionType::Ep) => "het Europees Parlement".to_string(),

        (ModelLocale::Fry, ElectionType::Tk) => {
            "de Twadde Keamer fan de Steaten-Generaal".to_string()
        }
        (ModelLocale::Fry, ElectionType::Ek) => {
            "de Earste Keamer fan de Steaten-Generaal".to_string()
        }
        (ModelLocale::Fry, ElectionType::Ps(province)) => {
            format!(
                "de Provinsjale Steaten fan {}",
                province.title(AnyLocale::from(locale))
            )
        }
        (ModelLocale::Fry, ElectionType::Ws(council)) => format!(
            "it algemien bestjoer fan it wetterskip {}",
            council.title(AnyLocale::from(locale))
        ),
        (ModelLocale::Fry, ElectionType::Ep) => "het Europees Parlement".to_string(),

        (_, ElectionType::Kc) => todo!("Support municipality regions"),
        (_, ElectionType::Er) => todo!("Support Island regions"),
        (_, ElectionType::Gr) => todo!("Support municipality regions"),
    }
}
