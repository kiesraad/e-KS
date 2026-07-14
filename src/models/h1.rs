//! Model H 1: Kandidatenlijst / Kandidatelist.

use textris_pdf::{
    build::{Text, Textris, cell, fill_in, italic, mono, text},
    theme::ColumnWidth::{Auto, Fraction},
};

use super::{
    Pdf,
    inputs::{ElectoralDistricts, ModelData, Person},
    layout::{
        bold_value_section, column_table, districts_section, signature_line, start_h_document,
        translator,
    },
};
use crate::{
    core::{ElectionType, ModelLocale, constants::DEFAULT_DATE_FORMAT},
    list_designation::ListDesignation,
};

#[derive(Debug)]
pub struct H1 {
    pub common: ModelData,
    pub electoral_districts: ElectoralDistricts,
    pub previously_seated: bool,
    pub list_designation: ListDesignation,
    pub list_submitter: Person,
    pub substitute_submitters: Vec<Person>,
}

impl Pdf for H1 {
    fn document(&self) -> Textris {
        let trans = translator(self.common.locale);
        let mut doc = start_h_document(
            &self.common,
            "Model H 1",
            trans("Kandidatenlijst", "Kandidatelist"),
            trans(
                "Met dit formulier stelt u, als inleveraar van de kandidatenlijst, kandidaten verkiesbaar voor een verkiezing.",
                "Mei dit formulier stelle jo, as dejinge dy’t de kandidatelist ynleveret, kandidaten ferkiesber foar in ferkiezing.",
            ),
            None,
            trans(
                "Het gaat om de verkiezing van ",
                "It giet om de ferkiezing fan ",
            ),
        );

        districts_section(
            &mut doc,
            self.common.locale,
            &self.electoral_districts,
            trans(
                "De kandidatenlijst wordt ingeleverd voor ",
                "De kandidatelist wurdt ynlevere foar ",
            ),
            trans("alle kieskringen.", "alle kiesrûnten."),
            Some(trans(
                "de volgende kieskring(en):",
                "de neikommende kiesrûnte(n):",
            )),
        );

        bold_value_section(
            &mut doc,
            trans(
                "Aanduiding van de politieke groepering",
                "De politike groepearring",
            ),
            trans(
                "Aanduiding boven de kandidatenlijst: ",
                "De politike groepearring dêr’t jo de kandidatelist fan stypje: ",
            ),
            &self.common.designation,
        );

        doc.h3_numbered(trans("Kandidaten op de lijst", "Kandidaten op de list"));
        doc.table_styled(
            &column_table([Auto, Fraction(1), Fraction(1), Fraction(1), Fraction(1)]),
            [
                "",
                trans("naam", "namme"),
                trans("voorletters", "foarletters"),
                trans("geboortedatum", "bertedatum"),
                trans("woonplaats", "wenplak"),
            ],
            self.common.candidates.iter().map(|c| {
                [
                    text(c.position.to_string()),
                    text(&c.last_name),
                    text(&c.initials),
                    mono(c.date_of_birth.format(DEFAULT_DATE_FORMAT).to_string()),
                    text(&c.locality),
                ]
            }),
        );

        doc.h3_numbered(trans(
            "Vervanger(s) voor het herstel van verzuimen",
            "Ferfanger(s) foar it ferhelpen fan fersommen",
        ));
        if self.substitute_submitters.is_empty() {
            doc.paragraph(italic(trans("geen", "geen")));
        } else {
            doc.table_styled(
                &column_table([
                    Auto,
                    Fraction(4),
                    Fraction(4),
                    Fraction(4),
                    Fraction(3),
                    Fraction(6),
                ]),
                [
                    "",
                    trans("naam", "namme"),
                    trans("voorletters", "foarletters"),
                    trans("postadres", "postadres"),
                    trans("postcode", "postkoade"),
                    trans("plaats", "plak"),
                ],
                self.substitute_submitters.iter().enumerate().map(|(i, s)| {
                    [
                        text((i + 1).to_string()),
                        text(&s.last_name),
                        text(&s.initials),
                        text(&s.postal_address.street_address),
                        mono(&s.postal_address.postal_code),
                        text(&s.postal_address.locality),
                    ]
                }),
            );
        }

        doc.h3_numbered(trans(
            "In te leveren bij de kandidatenlijst",
            "Yn te leverjen by de kandidatelist",
        ));
        doc.paragraph(trans(
            "Ik ben verplicht de volgende bijlage(n) in te leveren bij de kandidatenlijst:",
            "Ik bin ferplichte de neikommende taheakke by de kandidatelist yn te leverjen:",
        ));
        doc.task_list(self.attachments().into_iter().map(|item| (true, item)));

        doc.h3_numbered(trans(
            "Ondertekening door de inleveraar",
            "Undertekening troch dejinge dy’t ynleveret",
        ));
        let submitter = &self.list_submitter;
        let address = &submitter.postal_address;
        doc.label_table([
            [
                cell(trans("Naam en voorletters", "Namme en foarletters")),
                cell(format!("{}, {}", submitter.last_name, submitter.initials)),
            ],
            [
                cell(trans(
                    "Postadres, postcode en plaats",
                    "Postadres, postkoade en plak",
                )),
                cell(format!(
                    "{}, {} {}",
                    address.street_address, address.postal_code, address.locality
                )),
            ],
            [cell(trans("Datum", "Datum")), fill_in()],
        ]);
        signature_line(&mut doc, trans("Handtekening", "Hantekening"));

        doc
    }

    fn filename(&self) -> String {
        match self.common.locale {
            ModelLocale::Nl => "h1-kandidatenlijst.pdf".to_string(),
            ModelLocale::Fry => "h1-kandidatelist.pdf".to_string(),
        }
    }
}

impl H1 {
    /// The checklist of attachments that must be handed in with this list.
    fn attachments(&self) -> Vec<Text> {
        let trans = translator(self.common.locale);
        let election_type = self.common.election_type;
        let mut items = Vec::new();

        if self.list_designation != ListDesignation::Blank {
            items.push(text(trans(
                "Een verklaring van de gemachtigde(n) van de politieke groepering(en) waarmee aan mij toestemming wordt gegeven om de aanduiding boven de kandidatenlijst te plaatsen, want ik heb een aanduiding boven de lijst geplaatst (model H 3-1 of H 3-2).",
                "In ferklearring fan de lêsthawwer(s) fan de politike groepearring(s) dêr’t my tastimming mei jûn wurdt om de oantsjutting boppe de kandidatelist te pleatsen, want ik haw in oantsjutting boppe de list pleatst (model H 3-1 of H 3-2).",
            )));
        }
        if !self.previously_seated {
            let form = if election_type == ElectionType::Kcni {
                "model Pa 11"
            } else {
                "model H 4"
            };
            items.push(text(format!(
                "{} ({form}).",
                trans(
                    "Verklaringen van kiezers dat zij de lijst ondersteunen, want de lijst komt niet in aanmerking voor de ontheffing van deze verplichting",
                    "Ferklearrings fan kiezers dat hja de list stypje, want de list komt net yn oanmerking foar de ûntheffing fan dy ferplichtings",
                )
            )));
        }
        items.push(text(trans(
            "Een verklaring van iedere op de lijst voorkomende kandidaat dat hij instemt met zijn kandidaatstelling op de lijst (model H 9).",
            "In ferklearring fan alle op de list foarkommende kandidaten dat se ynstimme mei harren kandidaatstelling op de list (model H 9).",
        )));
        items.push(text(trans(
            "Een kopie van een geldig identiteitsbewijs van iedere kandidaat die géén zitting heeft in het orgaan waarvoor de verkiezing wordt gehouden.",
            "In kopy fan in jildich identiteitsbewiis fan alle kandidaten dy’t gjin sit hawwe yn it orgaan dêr’t de ferkiezing foar hâlden wurdt.",
        )));
        if !self.previously_seated {
            items.push(text(trans(
                "Een betalingsbewijs van de waarborgsom, want de lijst komt niet in aanmerking voor de ontheffing van deze verplichting (model H 12).",
                "In betellingsbewiis fan de boarchsom, want de list komt net yn oanmerking foar de ûntheffing fan dy ferplichting (model H 12).",
            )));
        }
        if matches!(
            election_type,
            ElectionType::Ps
                | ElectionType::Ws
                | ElectionType::Gr
                | ElectionType::Er
                | ElectionType::Kc
        ) {
            items.push(text(trans(
                "Een verklaring van voorgenomen vestiging van iedere op de lijst voorkomende kandidaat die niet woonachtig is in het gebied waarop de verkiezing betrekking heeft (alleen bij een verkiezing van provinciale staten, het algemeen bestuur van een waterschap, een gemeenteraad, de eilandsraden van de openbare lichamen Bonaire, Saba of Sint Eustatius en de kiescolleges van de openbare lichamen).",
                "In ferklearring fan foarnommen fêstiging foar alle op de list foarkommende kandidaten dy’t net wenjend binne yn it gebiet dêr’t de ferkiezing op slacht (allinnich by in ferkiezing fan provinsjale steaten, it algemien bestjoer fan in wetterskip, in gemeenteried, de eilânrieden fan it iepenbiere lichem Bonêre, Saba of Sint Eustaasjus en de kieskolleezjes fan it iepenbiere lichem).",
            )));
        }
        if election_type == ElectionType::Kcni {
            items.push(text(trans(
                "Een verklaring van voorgenomen vestiging buiten Nederland van iedere op de lijst voorkomende kandidaat die woonachtig is in Nederland (alleen bij een verkiezing van het kiescollege voor niet-ingezetenen).",
                "In ferklearring fan foarnommen fêstiging bûten Nederlân fan elke op de list foarkommende kandidaat dy’t yn Nederlân wennet (allinnich by in ferkiezing fan it kieskolleezje foar net-ynwenners).",
            )));
        }
        if election_type == ElectionType::Ep {
            items.push(text(trans(
                "Een verklaring van iedere op de lijst voorkomende kandidaat dat hij niet in een andere lidstaat kandidaat zal zijn voor het Europees Parlement (model Y 13).",
                "In ferklearring fan alle op de list foarkommende kandidaten dat se foar it Europeeske Parlemint net yn in oare lidsteat kandidaat wêze sille (model Y 13).",
            )));
            items.push(text(trans(
                "Een verklaring van kandidaten die onderdaan zijn van een andere lidstaat, dat zij in die lidstaat niet zijn uitgesloten van het recht om gekozen te worden voor de verkiezingen van het Europees Parlement (model Y 35).",
                "In ferklearring fan kandidaten dy’t ûnderdaan binne fan in oare lidsteat, dat sy yn dy lidsteat net útsletten binne fan it rjocht om keazen te wurden foar de ferkiezings fan it Europeeske Parlemint (model Y 35).",
            )));
        }
        items
    }
}
