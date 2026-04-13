use serde::Serialize;

use crate::Locale;

#[derive(Debug, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ElectionType {
    Tk,
    Ek,
    Gr,
    Ps,
    Ws,
    Ep,
    Kc, // TODO typst templates now use KCNI as type for "Kies College Niet Ingezetenen", make sure this is properly supported
    Er,
}

impl ElectionType {
    pub fn title(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (ElectionType::Tk, Locale::En) => "election of the House of Representatives",
            (ElectionType::Tk, Locale::Nl) => "Tweede Kamerverkiezing",
            (ElectionType::Ek, Locale::En) => "election of the Senate",
            (ElectionType::Ek, Locale::Nl) => "Eerste Kamerverkiezing",
            (ElectionType::Gr, Locale::En) => "elections of the municipal council",
            (ElectionType::Gr, Locale::Nl) => "gemeenteraadsverkiezingen",
            (ElectionType::Ps, Locale::En) => "elections of the provincial council",
            (ElectionType::Ps, Locale::Nl) => "Provinciale Statenverkiezingen",
            (ElectionType::Ws, Locale::En) => "elections of the water authority",
            (ElectionType::Ws, Locale::Nl) => "waterschapsverkiezingen",
            (ElectionType::Ep, Locale::En) => "election of the European Parliament",
            (ElectionType::Ep, Locale::Nl) => "Europees Parlementsverkiezing",
            (ElectionType::Kc, Locale::En) => "electoral colleges for the Senate",
            (ElectionType::Kc, Locale::Nl) => "kiescolleges Eerste Kamer",
            (ElectionType::Er, Locale::En) => "elections of the Island Councils",
            (ElectionType::Er, Locale::Nl) => "eilandsraadsverkiezingen",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn election_type_title_match() {
        for election_type in &[
            ElectionType::Tk,
            ElectionType::Ek,
            ElectionType::Gr,
            ElectionType::Ps,
            ElectionType::Ws,
            ElectionType::Ep,
            ElectionType::Er,
        ] {
            assert!(election_type.title(Locale::Nl).contains("verkiezing"));
            assert!(election_type.title(Locale::En).contains("election"));
        }

        // Kc uses "kiescolleges" rather than "verkiezing"
        assert!(ElectionType::Kc.title(Locale::Nl).contains("kiescolleges"));
        assert!(
            ElectionType::Kc
                .title(Locale::En)
                .contains("electoral colleges")
        );
    }
}
