use serde::Serialize;

use crate::Locale;

#[derive(Debug, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ElectionType {
    Ek,
    Tk,
}

impl ElectionType {
    pub fn title(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (ElectionType::Ek, Locale::En) => "election of the Senate",
            (ElectionType::Ek, Locale::Nl) => "Eerste Kamerverkiezing",
            (ElectionType::Tk, Locale::En) => "election of the House of Representatives",
            (ElectionType::Tk, Locale::Nl) => "Tweede Kamerverkiezing",
        }
    }
}
