//! Model H 4: Ondersteuningsverklaring / Stipeferklearring.

use serde::Deserialize;
use textris_pdf::{
    build::{Textris, cell, fill_in, text},
    theme::ColumnWidth::{Auto, Fraction},
};

use super::{
    Pdf,
    inputs::{ModelData, ModelElectionType},
    layout::{column_table, signature_line, start_document, translator, warning},
};
use crate::core::ModelLocale;

#[derive(Debug, Deserialize)]
pub struct H4 {
    #[serde(flatten)]
    pub common: ModelData,
}

impl Pdf for H4 {
    fn filename(&self) -> String {
        match self.common.locale {
            ModelLocale::Nl => "h4-ondersteuningsverklaring.pdf".to_string(),
            ModelLocale::Fry => "h4-stipeferklearring.pdf".to_string(),
        }
    }

    fn document(&self) -> Textris {
        let trans = translator(self.common.locale);
        let mut doc = start_document(
            "Model H 4",
            trans("Ondersteuningsverklaring", "Stipeferklearring"),
            self.common.locale,
            Some((self.common.event_id, &self.common.sha_hash)),
        );
        doc.paragraph(trans(
            "Met dit formulier verklaart u dat u een kandidatenlijst ondersteunt van een politieke groepering. Dit betekent dat u de deelname van de betreffende groepering aan de verkiezing mogelijk maakt. Deze verklaring wordt ter inzage gelegd.",
            "Mei dit formulier ferklearje jo dat jo in kandidatelist fan in politike groepearring stypje. Dat betsjut dat jo de dielname fan de oanbelangjende groepearring oan de ferkiezing mooglik meitsje. Dizze ferklearring wurdt op ynsjen lein.",
        ));
        warning(
            &mut doc,
            trans("Let op!", "Tink der om!"),
            trans(
                "U mag zich niet laten omkopen tot het afleggen van deze ondersteuningsverklaring. Degene die u omkoopt of u hiertoe anderszins dwingt, is tevens strafbaar. Op beide misdrijven staat een gevangenisstraf van maximaal zes maanden of een geldboete.",
                "Jo meie jo net omkeapje litte ta it ôflizzen fan dizze stipeferklearring. Dejinge dy't jo omkeapet of jo dêrta op oare wize twingt, is tagelyk strafber. Op beide misdriuwen stiet in finzenisstraf fan maksimaal seis moannen of in jildboete.",
            ),
        );

        doc.h3_numbered(trans("Verkiezing", "Ferkiezing"));
        doc.paragraph(
            text(trans(
                "Het gaat om de verkiezing van: ",
                "It giet om de ferkiezing fan: ",
            ))
            .bold(&self.common.election_name),
        );

        doc.h3_numbered(trans(
            "Aanduiding van de politieke groepering",
            "Oantsjutting fan de politike groepearring",
        ));
        doc.paragraph(
            text(trans(
                "De aanduiding van de politieke groepering waarvan u de kandidatenlijst ondersteunt: ",
                "De oantsjutting fan de politike groepearring dêr't jo de kandidatelist fan stypje: ",
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

        doc.h3_numbered(trans(
            "Ondertekening door de kiezer",
            "Undertekening troch de kiezer",
        ));
        doc.paragraph(trans(
            "Ik verklaar dat ik de bovengenoemde kandidatenlijst ondersteun.",
            "Ik ferklearje dat ik de boppeneamde kandidatelist stypje.",
        ));
        doc.label_table([
            [cell(trans("Datum", "Datum")), fill_in()],
            [cell(trans("Naam", "Namme")), fill_in()],
        ]);
        signature_line(&mut doc, trans("Handtekening", "Hantekening"));

        if self.common.election_type != ModelElectionType::Ek {
            self.mayor_section(&mut doc);
        }

        doc
    }
}

impl H4 {
    /// Word choice depending on whether the voter register is kept by a
    /// municipality (`gr`) or a public body (`non_gr`), like `is_municipality`
    /// in the Typst template.
    fn municipality<'a>(&self, gr: &'a str, non_gr: &'a str) -> String {
        match self.common.election_type {
            ModelElectionType::Er => non_gr.to_string(),
            ModelElectionType::Tk => format!("{gr} / {non_gr}"),
            _ => gr.to_string(),
        }
    }

    /// The mayor's statement that the supporter is a registered voter.
    fn mayor_section(&self, doc: &mut Textris) {
        let trans = translator(self.common.locale);
        let mayor = match self.common.locale {
            ModelLocale::Nl => self.municipality("burgemeester", "gezaghebber"),
            ModelLocale::Fry => self.municipality("boargemaster", "gesachhawwer"),
        };
        let municipality = match self.common.locale {
            ModelLocale::Nl => self.municipality("gemeente", "openbaar lichaam"),
            ModelLocale::Fry => self.municipality("gemeente", "iepenbier lichem"),
        };

        doc.h3_numbered(format!(
            "{} {mayor}",
            trans("Verklaring van de", "Ferklearring fan de")
        ));
        // textris-pdf has no inline fill-in within a paragraph; a run of
        // underscores approximates the blank line of the original template.
        doc.paragraph(
            text(format!("De {mayor} {} ", trans("van", "fan")))
                .normal("_".repeat(30))
                .normal(format!(
                    " {}",
                    trans(
                        "verklaart dat de ondersteuner in zijn {} als kiezer is geregistreerd.",
                        "ferklearret dat de stiper yn syn {} as kiezer registrearre is.",
                    )
                    .replace("{}", &municipality)
                )),
        );
        doc.label_table([
            [
                cell(trans(
                    "De kiezer behoort tot kieskring",
                    "De kiezer heart ta kiesrûnte",
                )),
                fill_in(),
            ],
            [cell(trans("Datum", "Datum")), fill_in()],
        ]);
        signature_line(
            doc,
            trans(
                "Ondertekening of gemeentestempel",
                "Undertekening of gemeentestimpel",
            ),
        );
    }
}
