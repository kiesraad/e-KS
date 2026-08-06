//! Example inputs for model I 1.

use super::strings;
use crate::models::{
    i1::I1,
    i4::{OmissionGroup, PublicSession},
};

fn i1_session() -> PublicSession {
    PublicSession {
        location: "'s-Gravenhage".to_string(),
        date: "5 april 2027".to_string(),
        time: "16:00 uur".to_string(),
        chair: "M.C. Voorzitter".to_string(),
        members: strings(&["A. Lid", "B. Lid", "C. Lid", "D. Lid", "E. Lid", "F. Lid"]),
    }
}

fn i1_found_omissions() -> Vec<OmissionGroup> {
    vec![
        OmissionGroup {
            designation: "De Geconstateerde Partij".to_string(),
            electoral_district: "kieskring 20 (Bonaire)".to_string(),
            omission_descriptions: strings(&[
                "Ten aanzien van kandidaat nr. 3 J. Altena ontbreekt de verklaring dat deze instemt met kandidaatstelling op de lijst.",
                "Ten aanzien van kandidaat nr. 12 G. Braber ontbreekt de verklaring dat deze instemt met kandidaatstelling op de lijst.",
            ]),
        },
        OmissionGroup {
            designation: "Kiesraad Demo 3".to_string(),
            electoral_district: "alle kieskringen".to_string(),
            omission_descriptions: strings(&[
                "Bij de lijst zijn niet voldoende geldige verklaringen van ondersteuning ingeleverd voor alle kieskringen.",
            ]),
        },
    ]
}

/// Both I 1 examples share the election and the session; only the omissions
/// found vary.
fn i1_example(found_omissions: Vec<OmissionGroup>) -> I1 {
    I1 {
        election_name: "de Eerste Kamer der Staten-Generaal".to_string(),
        election_date: "24 mei 2027".to_string(),
        session: i1_session(),
        found_omissions,
    }
}

pub fn i1_example_1() -> I1 {
    i1_example(i1_found_omissions())
}

/// No omissions found: the "geen verzuimen" fallback.
pub fn i1_example_2() -> I1 {
    i1_example(Vec::new())
}
