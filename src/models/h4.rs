//! Model H 4: Ondersteuningsverklaring / Stipeferklearring.

use textris_pdf::build::{Textris, cell, fill_in, text};

use super::{
    Pdf,
    inputs::{ModelData, ModelElectionType},
    layout::{
        candidates_section, election_section, signature_line, start_versioned, translator,
        warning_let_op,
    },
};
use crate::core::ModelLocale;

#[derive(Debug)]
pub struct H4 {
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
        let mut doc = start_versioned(
            "Model H 4",
            trans("Ondersteuningsverklaring", "Stipeferklearring"),
            &self.common,
        );
        doc.paragraph(trans(
            "Met dit formulier verklaart u dat u een kandidatenlijst ondersteunt van een politieke groepering. Dit betekent dat u de deelname van de betreffende groepering aan de verkiezing mogelijk maakt. Deze verklaring wordt ter inzage gelegd.",
            "Mei dit formulier ferklearje jo dat jo in kandidatelist fan in politike groepearring stypje. Dat betsjut dat jo de dielname fan de oanbelangjende groepearring oan de ferkiezing mooglik meitsje. Dizze ferklearring wurdt op ynsjen lein.",
        ));
        warning_let_op(
            &mut doc,
            self.common.locale,
            "U mag zich niet laten omkopen tot het afleggen van deze ondersteuningsverklaring. Degene die u omkoopt of u hiertoe anderszins dwingt, is tevens strafbaar. Op beide misdrijven staat een gevangenisstraf van maximaal zes maanden of een geldboete.",
            "Jo meie jo net omkeapje litte ta it ôflizzen fan dizze stipeferklearring. Dejinge dy't jo omkeapet of jo dêrta op oare wize twingt, is tagelyk strafber. Op beide misdriuwen stiet in finzenisstraf fan maksimaal seis moannen of in jildboete.",
        );

        election_section(
            &mut doc,
            self.common.locale,
            "Het gaat om de verkiezing van: ",
            "It giet om de ferkiezing fan: ",
            &self.common.election_name,
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

        candidates_section(&mut doc, self.common.locale, &self.common.candidates);

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
