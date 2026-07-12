//! Model H 3: authorisation to place a designation above a candidate list.
//! Depending on the list designation this renders as H 3-1 (a single
//! registered designation, [`super::h3_1`]) or H 3-2 (a combined designation,
//! [`super::h3_2`]). The shared sections live here.

use serde::Deserialize;
use textris_pdf::{
    build::{Textris, cell, fill_in, text},
    theme::ColumnWidth::{Auto, Fraction},
};

use super::{
    Pdf, h3_1, h3_2,
    inputs::{ElectoralDistricts, ModelData, NameAuthorisation, Person},
    layout::{column_table, signature_line, translator},
};
use crate::{core::ModelLocale, list_designation::ListDesignation};

/// Anchor of the designation section, referenced from the permission text.
pub(super) const DESIGNATION_SECTION: &str = "aanduiding";

#[derive(Debug, Deserialize)]
pub struct H3 {
    #[serde(flatten)]
    pub common: ModelData,
    pub electoral_districts: ElectoralDistricts,
    pub list_designation: ListDesignation,
    pub list_submitter: Person,
    pub name_authorisations: Vec<NameAuthorisation>,
}

impl Pdf for H3 {
    fn filename(&self) -> String {
        match (self.common.locale, self.list_designation) {
            (ModelLocale::Nl, ListDesignation::Combined) => "h3-2-samengevoegde-aanduiding.pdf",
            (ModelLocale::Fry, ListDesignation::Combined) => "h3-2-gearfoege-oantsjutting.pdf",
            (ModelLocale::Nl, _) => "h3-1-aanduiding.pdf",
            (ModelLocale::Fry, _) => "h3-1-oantsjutting.pdf",
        }
        .to_string()
    }

    fn document(&self) -> Textris {
        if self.list_designation == ListDesignation::Combined {
            h3_2::document(self)
        } else {
            h3_1::document(self)
        }
    }
}

/// The numbered "Verkiezing" section.
pub(super) fn election_section(doc: &mut Textris, input: &H3) {
    let trans = translator(input.common.locale);
    doc.h3_numbered(trans("Verkiezing", "Ferkiezing"));
    doc.paragraph(
        text(trans(
            "Het gaat om de kandidatenlijst voor de verkiezingen van: ",
            "It giet om de kandidatelist foar de ferkiezing fan: ",
        ))
        .bold(&input.common.election_name),
    );
}

/// The numbered "Kieskringen" section, omitted for single-district elections.
pub(super) fn districts_section(doc: &mut Textris, input: &H3) {
    let trans = translator(input.common.locale);
    if input.electoral_districts == ElectoralDistricts::OnlyOne {
        return;
    }
    doc.h3_numbered(trans("Kieskringen", "Kiesrûnten"));
    let intro = text(trans("De machtiging geldt ", "De machtiging jildt "));
    match &input.electoral_districts {
        ElectoralDistricts::All => {
            doc.paragraph(intro.bold(trans(
                "voor alle kieskringen waarvoor de kandidatenlijst wordt ingeleverd.",
                "foar alle kiesrûnten dêr’t de kandidatelist foar ynlevere wurdt.",
            )));
        }
        ElectoralDistricts::Some(districts) => {
            doc.paragraph(intro.bold(trans(
                "uitsluitend voor de volgende kieskring(en):",
                "allinnich foar de neikommende kiesrûnte(n):",
            )));
            doc.paragraph(districts.join(", "));
        }
        ElectoralDistricts::OnlyOne => {}
    }
}

/// The numbered "Toestemming aan de inleveraar" section. `we` selects the
/// plural wording of H 3-2. The running text refers back to the designation
/// section by its number.
pub(super) fn permission_section(doc: &mut Textris, input: &H3, we: bool) {
    let trans = translator(input.common.locale);
    doc.h3_numbered(trans(
        "Toestemming aan de inleveraar",
        "Tastimming oan dejinge dy’t ynleveret",
    ));
    let intro = if we {
        trans("Wij geven toestemming aan ", "Wy jouwe tastimming oan ")
    } else {
        trans("Ik geef toestemming aan ", "Ik jou tastimming oan ")
    };
    let submitter = &input.list_submitter;
    doc.paragraph(
        text(intro)
            .bold(format!("{}, {}", submitter.last_name, submitter.initials))
            .normal(trans(" om de onder punt ", " om de ûnder punt "))
            .section_ref(DESIGNATION_SECTION)
            .normal(trans(
                " vermelde aanduiding boven de kandidatenlijst te plaatsen.",
                " neamde oantsjutting boppe de kandidatelist te pleatsen.",
            )),
    );
}

/// The numbered "Kandidaten op de lijst" section.
pub(super) fn candidates_section(doc: &mut Textris, input: &H3) {
    let trans = translator(input.common.locale);
    doc.h3_numbered(trans("Kandidaten op de lijst", "Kandidaten op de list"));
    doc.table_styled(
        &column_table([Auto, Fraction(1), Fraction(1), Fraction(1)]),
        [
            "",
            trans("naam", "namme"),
            trans("voorletters", "foarletters"),
            trans("woonplaats", "wenplak"),
        ],
        input.common.candidates.iter().map(|c| {
            [
                text(c.position.to_string()),
                text(&c.last_name),
                text(&c.initials),
                text(&c.locality),
            ]
        }),
    );
}

/// The label rows and signature line for one authorised representative.
pub(super) fn authorisation_signature(
    doc: &mut Textris,
    locale: ModelLocale,
    authorisation: &NameAuthorisation,
) {
    let trans = translator(locale);
    let name = [
        authorisation.last_name.as_str(),
        authorisation.initials.as_str(),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(", ");

    doc.label_table([
        [cell(trans("Datum", "Datum")), fill_in()],
        [
            cell(trans(
                "Naam van de gemachtigde van de politieke groepering",
                "Namme fan de lêsthawwer fan de politike groepearring",
            )),
            cell(name),
        ],
        [
            cell(trans(
                "Volledige statutaire naam van de politieke groepering",
                "Folsleine statutêre namme fan de politike groepearring",
            )),
            cell(&*authorisation.legal_name),
        ],
    ]);
    signature_line(doc, trans("Handtekening", "Hantekening"));
}
