//! Example inputs for model H 4.

use super::{full_candidates, model_data};
use crate::{
    core::{ElectionType, ModelLocale},
    models::h4::H4,
};

/// All H 4 examples share the appellation and candidates; only the election
/// and the locale vary.
fn h4_example(election_name: &str, election_type: ElectionType, locale: ModelLocale) -> H4 {
    H4 {
        common: model_data(
            election_name,
            election_type,
            "Test Partij (TP)",
            full_candidates(),
            locale,
        ),
    }
}

pub fn h4_example_1() -> H4 {
    h4_example(
        "de Eerste Kamer der Staten-Generaal",
        ElectionType::Ek,
        ModelLocale::Nl,
    )
}

pub fn h4_example_2() -> H4 {
    h4_example(
        "de gemeenteraad van Amsterdam",
        ElectionType::Gr,
        ModelLocale::Nl,
    )
}

pub fn h4_example_3() -> H4 {
    h4_example(
        "de Twadde Keamer fan de Steaten-Generaal",
        ElectionType::Tk,
        ModelLocale::Fry,
    )
}
