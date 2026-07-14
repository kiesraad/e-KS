//! Model H 9: Instemmingsverklaring / Ynstimmingsferklearring.

use textris_pdf::{
    build::{Textris, cell, fill_in, italic, mono, text},
    theme::ColumnWidth::Fraction,
};

use super::{
    Pdf,
    inputs::{DetailedCandidate, ElectoralDistricts, ModelData},
    layout::{
        bold_value_section, candidates_section, column_table, districts_section, signature_line,
        start_h_document, translator,
    },
};
use crate::{core::ElectionType, utils::slugify_teletex};

#[derive(Debug)]
pub struct H9 {
    pub common: ModelData,
    pub electoral_districts: ElectoralDistricts,
    pub detailed_candidate: DetailedCandidate,
}

impl Pdf for H9 {
    fn document(&self) -> Textris {
        let trans = translator(self.common.locale);
        let mut doc = start_h_document(
            &self.common,
            "Model H 9",
            trans("Instemmingsverklaring", "Ynstimmingsferklearring"),
            trans(
                "Met dit formulier stemt u ermee in dat u op onderstaande kandidatenlijst staat, en u stemt in met uw positie op die lijst.",
                "Mei dit formulier stimme jo dermei yn dat jo op ûndersteande kandidatelist steane en jo ynstimme mei jo posysje op dy list.",
            ),
            Some((
                trans("Let op!", "Tink der om!"),
                trans(
                    "Bent u nog geen lid van het vertegenwoordigend orgaan? Voeg dan een kopie van een geldig identiteitsbewijs bij.",
                    "Binne jo noch gjin lid fan it fertsjintwurdigjend orgaan? Foegje dan in kopy fan in jildich identiteitsbewiis ta.",
                ),
            )),
            trans(
                "Het gaat om de verkiezing van ",
                "It giet om de ferkiezing fan ",
            ),
        );

        // NOTE: This text slightly differs from the reference H9 but is
        // confirmed to be legal. Single-district elections omit the section.
        districts_section(
            &mut doc,
            self.common.locale,
            &self.electoral_districts,
            trans(
                "Mijn instemming geldt voor: ",
                "Myn ynstimming jildt foar: ",
            ),
            trans("alle kieskringen", "alle kiesrûnten"),
            None,
        );

        bold_value_section(
            &mut doc,
            trans("Politieke groepering", "Politike groepearring"),
            trans(
                "De aanduiding van de politieke groepering waarvan de kandidatenlijst is: ",
                "De oantsjutting fan de politike groepearring dêr’t de kandidatelist fan is: ",
            ),
            &self.common.designation,
        );

        candidates_section(&mut doc, self.common.locale, &self.common.candidates);

        let candidate = &self.detailed_candidate;

        if let Some(representative) = &candidate.representative {
            doc.h3_numbered(trans(
                "Gemachtigde voor het aannemen van uw benoeming",
                "Lêsthawwer foar it oannimmen fan jo beneaming",
            ));
            doc.table_styled(
                &column_table([
                    Fraction(4),
                    Fraction(4),
                    Fraction(4),
                    Fraction(3),
                    Fraction(6),
                ]),
                [
                    trans("naam", "namme"),
                    trans("voorletters", "foarletters"),
                    trans("postadres", "postadres"),
                    trans("postcode", "postkoade"),
                    trans("plaats", "plak"),
                ],
                [[
                    text(&representative.last_name),
                    text(&representative.initials),
                    text(&representative.postal_address.street_address),
                    mono(&representative.postal_address.postal_code),
                    text(&representative.postal_address.locality),
                ]],
            );
        }

        if self.common.election_type != ElectionType::Kcni && candidate.representative.is_none() {
            doc.h3_numbered(trans(
                "Adres voor de kennisgeving van mijn benoeming",
                "Adres foar de meidieling fan myn beneaming",
            ));
            // this section does not apply to the election of the electoral
            // college for non-residents
            match &candidate.postal_address {
                None => {
                    doc.paragraph(italic(trans("niet van toepassing", "net fan tapassing")));
                }
                Some(address) => {
                    doc.table_styled(
                        &column_table([Fraction(2), Fraction(1), Fraction(2)]),
                        [
                            trans("postadres", "postadres"),
                            trans("postcode", "postkoade"),
                            trans("plaats", "plak"),
                        ],
                        [[
                            text(&address.street_address),
                            mono(&address.postal_code),
                            text(&address.locality),
                        ]],
                    );
                }
            }
        }

        if self.common.election_type == ElectionType::Kcni && candidate.representative.is_none() {
            doc.h3_numbered(trans(
                "Kennisgeving van mijn benoeming ontvangen langs digitale weg",
                "Kennisjouwing fan myn beneaming fia digitale wei tasjoerd krije",
            ));
            doc.task_list([(
                false,
                trans(
                    "Ik stem ermee in de kennisgeving van mijn benoeming te ontvangen via een digitale berichtenbox waartoe ik toegang kan krijgen met gebruikmaking van een DigiD. Hierbij bevestig ik tevens dat ik een DigiD zal aanvragen indien ik hier nog niet over beschik.",
                    "Ik stim dermei yn dat de kennisjouwing fan myn beneaming my tastjoerd wurdt fia in digitale berjochteboks dêr’t ik tagong ta krije kin mei in DigiD. Ek befêstigje ik dat ik in DigiD oanfreegje sil as ik dy noch net ha.",
                ),
            )]);
        }

        doc.h3_numbered(trans(
            "Ondertekening door de kandidaat",
            "Undertekening troch de kandidaat",
        ));
        doc.label_table([
            [
                cell(trans("Naam", "Namme")),
                cell(format!(
                    "{}, {}",
                    candidate.candidate.last_name, candidate.initials_no_gender
                )),
            ],
            [
                cell(trans("Woonplaats", "Wenplak")),
                cell(&*candidate.candidate.locality),
            ],
            [
                cell(trans("Burgerservicenummer", "Boargerservicenûmer")),
                cell(candidate.bsn.as_deref().unwrap_or_default()),
            ],
            [cell(trans("Datum", "Datum")), fill_in()],
        ]);
        signature_line(&mut doc, trans("Handtekening", "Hantekening"));

        doc
    }

    fn filename(&self) -> String {
        format!(
            "h9-{}-{}.pdf",
            slugify_teletex(&self.detailed_candidate.candidate.last_name, true),
            self.detailed_candidate.candidate.position
        )
    }
}
