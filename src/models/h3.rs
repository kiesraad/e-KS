//! Model H 3: authorisation to place a designation above a candidate list.
//! Depending on the list designation this renders as H 3-1 (a single
//! registered designation) or H 3-2 (a combined designation). The two variants
//! share their structure; only the wording and the signing block differ.

use textris_pdf::build::{Textris, cell, fill_in, text};

use super::{
    Pdf,
    inputs::{ElectoralDistricts, ModelData, NameAuthorisation, Person},
    layout::{
        bold_value_section, candidates_section, districts_section, signature_line, start_document,
        translator,
    },
};
use crate::{core::ModelLocale, list_designation::ListDesignation};

/// Anchor of the designation section, referenced from the permission text.
const DESIGNATION_SECTION: &str = "aanduiding";

#[derive(Debug)]
pub struct H3 {
    pub common: ModelData,
    pub electoral_districts: ElectoralDistricts,
    pub list_designation: ListDesignation,
    pub list_submitter: Person,
    pub name_authorisations: Vec<NameAuthorisation>,
}

impl Pdf for H3 {
    /// H 3-1 (registered designation) and H 3-2 (combined designation) share
    /// their layout; `combined` selects the H 3-2 wording and its
    /// per-representative signing block.
    fn document(&self) -> Textris {
        let trans = translator(self.common.locale);
        let combined = self.list_designation == ListDesignation::Combined;

        let mut doc = start_document(
            if combined {
                "Model H 3-2"
            } else {
                "Model H 3-1"
            },
            if combined {
                trans(
                    "Machtiging om samengevoegde aanduiding boven kandidatenlijst te plaatsen",
                    "Machtiging om gearfoege oantsjutting boppe kandidatelist te pleatsen",
                )
            } else {
                trans(
                    "Machtiging om aanduiding boven kandidatenlijst te plaatsen",
                    "Machtiging om oantsjutting boppe kandidatelist te pleatsen",
                )
            },
            self.common.locale,
            Some((self.common.event_id, &self.common.sha_hash)),
        );
        doc.paragraph(if combined {
            trans(
                "Met dit formulier geeft u de inleveraar van de kandidatenlijst toestemming om een aanduiding boven de kandidatenlijst te plaatsen, die is gevormd door samenvoeging van de aanduidingen van politieke groeperingen of afkortingen daarvan.",
                "Mei dit formulier jouwe jo dejinge dy’t de kandidatelist ynleveret tastimming om in oantsjutting boppe de kandidatelist te pleatsen, dy’t foarme is troch gearfoeging fan de oantsjuttings fan politike groepearrings of ôfkoartings dêrfan.",
            )
        } else {
            trans(
                "Met dit formulier geeft u de inleveraar van de kandidatenlijst toestemming om de aanduiding die door uw politieke groepering is geregistreerd boven de kandidatenlijst te plaatsen.",
                "Mei dit formulier jouwe jo dejinge dy’t de kandidatelist ynleveret tastimming om de oantsjutting dy’t troch jo politike groepearring registrearre is boppe de kandidatelist te pleatsen.",
            )
        });
        doc.paragraph(trans(
            "U kunt alleen toestemming geven als u hiertoe gemachtigd bent door uw politieke groepering.",
            "Jo kinne allinnich tastimming jaan as jo dêrta machtige binne troch jo politike groepearring.",
        ));

        bold_value_section(
            &mut doc,
            trans("Verkiezing", "Ferkiezing"),
            trans(
                "Het gaat om de kandidatenlijst voor de verkiezingen van: ",
                "It giet om de kandidatelist foar de ferkiezing fan: ",
            ),
            &self.common.election_name,
        );

        districts_section(
            &mut doc,
            self.common.locale,
            &self.electoral_districts,
            trans("De machtiging geldt ", "De machtiging jildt "),
            trans(
                "voor alle kieskringen waarvoor de kandidatenlijst wordt ingeleverd.",
                "foar alle kiesrûnten dêr’t de kandidatelist foar ynlevere wurdt.",
            ),
            Some(trans(
                "uitsluitend voor de volgende kieskring(en):",
                "allinnich foar de neikommende kiesrûnte(n):",
            )),
        );

        doc.h3_numbered(if combined {
            trans(
                "Aanduiding van de politieke groeperingen",
                "Oantsjutting fan de politike groepearrings",
            )
        } else {
            trans(
                "Aanduiding van de politieke groepering",
                "Oantsjutting fan de politike groepearring",
            )
        })
        .anchor(DESIGNATION_SECTION);
        doc.paragraph(
            text(if combined {
                trans(
                    "De samengevoegde aanduiding van de politieke groeperingen: ",
                    "De gearfoege oantsjutting fan de politike groepearrings: ",
                )
            } else {
                trans(
                    "De geregistreerde aanduiding van de politieke groepering: ",
                    "De registrearre oantsjutting fan de politike groepearring: ",
                )
            })
            .bold(&self.common.designation),
        );

        permission_section(&mut doc, self, combined);
        candidates_section(&mut doc, self.common.locale, &self.common.candidates);

        if combined {
            doc.h3_numbered(trans(
                "Ondertekening door de gemachtigden",
                "Undertekening troch de lêsthawwer",
            ));
            for (index, authorisation) in self.name_authorisations.iter().enumerate() {
                doc.h4(format!(
                    "{} {}",
                    trans(
                        "Gemachtigde van politieke groepering",
                        "Lêsthawwer fan politike groepearring",
                    ),
                    index + 1
                ));
                authorisation_signature(&mut doc, self.common.locale, authorisation);
            }
        } else {
            doc.h3_numbered(trans(
                "Ondertekening door de gemachtigde van de politieke groepering",
                "Undertekening troch de lêsthawwer fan de politike groepearring",
            ));
            if let Some(authorisation) = self.name_authorisations.first() {
                authorisation_signature(&mut doc, self.common.locale, authorisation);
            }
        }

        doc
    }

    fn filename(&self) -> String {
        match (self.common.locale, self.list_designation) {
            (ModelLocale::Nl, ListDesignation::Combined) => "h3-2-samengevoegde-aanduiding.pdf",
            (ModelLocale::Fry, ListDesignation::Combined) => "h3-2-gearfoege-oantsjutting.pdf",
            (ModelLocale::Nl, _) => "h3-1-aanduiding.pdf",
            (ModelLocale::Fry, _) => "h3-1-oantsjutting.pdf",
        }
        .to_string()
    }
}

/// The numbered "Toestemming aan de inleveraar" section. `we` selects the
/// plural wording of H 3-2. The running text refers back to the designation
/// section by its number.
fn permission_section(doc: &mut Textris, input: &H3, we: bool) {
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

/// The label rows and signature line for one authorised representative.
fn authorisation_signature(
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
