//! Model H 9: Instemmingsverklaring / Ynstimmingsferklearring.

use serde::Deserialize;
use textris_pdf::{
    build::{Textris, cell, fill_in, italic, mono, text},
    theme::ColumnWidth::{Auto, Fraction},
};

use super::{
    Pdf,
    inputs::{DetailedCandidate, ElectoralDistricts, ModelData, ModelElectionType},
    layout::{column_table, signature_line, start_document, translator, warning},
};
use crate::utils::slugify_teletex;

#[derive(Debug, Deserialize)]
pub struct H9 {
    #[serde(flatten)]
    pub common: ModelData,
    pub electoral_districts: ElectoralDistricts,
    pub detailed_candidate: DetailedCandidate,
}

impl Pdf for H9 {
    fn filename(&self) -> String {
        format!(
            "h9-{}-{}.pdf",
            slugify_teletex(&self.detailed_candidate.candidate.last_name, true),
            self.detailed_candidate.candidate.position
        )
    }

    fn document(&self) -> Textris {
        let trans = translator(self.common.locale);
        let mut doc = start_document(
            "Model H 9",
            trans("Instemmingsverklaring", "Ynstimmingsferklearring"),
            self.common.locale,
            Some((self.common.event_id, &self.common.sha_hash)),
        );
        doc.paragraph(trans(
            "Met dit formulier stemt u ermee in dat u op onderstaande kandidatenlijst staat, en u stemt in met uw positie op die lijst.",
            "Mei dit formulier stimme jo dermei yn dat jo op ûndersteande kandidatelist steane en jo ynstimme mei jo posysje op dy list.",
        ));
        warning(
            &mut doc,
            trans("Let op!", "Tink der om!"),
            trans(
                "Bent u nog geen lid van het vertegenwoordigend orgaan? Voeg dan een kopie van een geldig identiteitsbewijs bij.",
                "Binne jo noch gjin lid fan it fertsjintwurdigjend orgaan? Foegje dan in kopy fan in jildich identiteitsbewiis ta.",
            ),
        );

        doc.h3_numbered(trans("Verkiezing", "Ferkiezing"));
        doc.paragraph(
            text(trans(
                "Het gaat om de verkiezing van ",
                "It giet om de ferkiezing fan ",
            ))
            .bold(&self.common.election_name),
        );

        if self.electoral_districts != ElectoralDistricts::OnlyOne {
            doc.h3_numbered(trans("Kieskringen", "Kiesrûnten"));
            // NOTE: This text slightly differs from the reference H9 but is
            // confirmed to be legal
            let intro = text(trans(
                "Mijn instemming geldt voor: ",
                "Myn ynstimming jildt foar: ",
            ));
            match &self.electoral_districts {
                ElectoralDistricts::All => {
                    doc.paragraph(intro.bold(trans("alle kieskringen", "alle kiesrûnten")));
                }
                ElectoralDistricts::Some(districts) => {
                    doc.paragraph(intro);
                    doc.paragraph(districts.join(", "));
                }
                ElectoralDistricts::OnlyOne => {}
            }
        }

        doc.h3_numbered(trans("Politieke groepering", "Politike groepearring"));
        doc.paragraph(
            text(trans(
                "De aanduiding van de politieke groepering waarvan de kandidatenlijst is: ",
                "De oantsjutting fan de politike groepearring dêr’t de kandidatelist fan is: ",
            ))
            .bold(&self.common.designation),
        );

        doc.h3_numbered(trans("Kandidaten op de lijst", "Kandidaten op de list"));
        doc.table_styled(
            &column_table([Auto, Fraction(1), Fraction(1), Fraction(1)]),
            [
                "",
                trans("naam", "namme"),
                trans("voorletters", "foarletters"),
                trans("woonplaats", "wenplak"),
            ],
            self.common.candidates.iter().map(|c| {
                [
                    text(c.position.to_string()),
                    text(&c.last_name),
                    text(&c.initials),
                    text(&c.locality),
                ]
            }),
        );

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

        if self.common.election_type != ModelElectionType::Kcni
            && candidate.representative.is_none()
        {
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

        if self.common.election_type == ModelElectionType::Kcni
            && candidate.representative.is_none()
        {
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
}
