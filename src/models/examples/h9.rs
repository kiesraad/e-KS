//! Example inputs for model H 9.

use super::{brief_candidates, candidate, date, model_data, postal_address, standard_districts};
use crate::{
    core::{ElectionType, ModelLocale},
    models::{
        h9::H9,
        inputs::{DetailedCandidate, ElectoralDistricts, Person},
    },
};

/// The resident detailed candidate of examples 1 and 3: no representative,
/// notified at her own postal address.
fn h9_resident_candidate() -> DetailedCandidate {
    DetailedCandidate {
        candidate: candidate("Akwasi", "M. (v)", date(1997, 9, 1), "Ede", 1),
        initials_no_gender: "M. (Maria)".to_string(),
        bsn: Some("999999321".to_string()),
        representative: None,
        needs_representative: false,
        postal_address: Some(postal_address("Molenweg 37", "1111AA", "Ede")),
    }
}

pub fn h9_example_1() -> H9 {
    H9 {
        common: model_data(
            "de Tweede Kamer der Staten-Generaal",
            ElectionType::Tk,
            "Een Andere Partij (EAP)",
            brief_candidates("Ede"),
            ModelLocale::Nl,
        ),
        electoral_districts: standard_districts(),
        detailed_candidate: h9_resident_candidate(),
    }
}

pub fn h9_example_2() -> H9 {
    H9 {
        common: model_data(
            "de Tweede Kamer der Staten-Generaal",
            ElectionType::Tk,
            "Een Andere Partij (EAP)",
            brief_candidates("Berlin"),
            ModelLocale::Fry,
        ),
        electoral_districts: ElectoralDistricts::All,
        detailed_candidate: DetailedCandidate {
            candidate: candidate("Akwasi", "M. (v)", date(1997, 12, 25), "Berlin", 1),
            initials_no_gender: "M. (Maria)".to_string(),
            bsn: Some("999999321".to_string()),
            representative: Some(Person {
                last_name: "Akwasi".to_string(),
                initials: "T.J.".to_string(),
                postal_address: postal_address("Molenweg 37", "1111AA", "Ede"),
            }),
            needs_representative: true,
            postal_address: None,
        },
    }
}

pub fn h9_example_3() -> H9 {
    H9 {
        common: model_data(
            "het algemeen bestuur van het waterschap Rijn en IJssel",
            ElectionType::Ws,
            "Een Andere Partij (EAP)",
            brief_candidates("Ede"),
            ModelLocale::Nl,
        ),
        electoral_districts: ElectoralDistricts::OnlyOne,
        detailed_candidate: h9_resident_candidate(),
    }
}
