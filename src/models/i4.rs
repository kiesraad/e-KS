//! Model I 4: Proces-verbaal over geldigheid en nummering kandidatenlijsten.
//! This model is Dutch-only.

use serde::Deserialize;
use textris_pdf::{
    build::{Textris, blank, cell, fill_in, text},
    model::ListMarker,
    theme::{
        Align,
        ColumnWidth::{Auto, Fraction},
        ColumnWidths, TableStyle, em,
    },
};

use super::{
    Pdf,
    layout::{column_table, column_table_aligned, plain_table, start_document},
};
use crate::core::ModelLocale;

#[derive(Debug, Deserialize)]
pub struct I4 {
    pub election_name: String,
    pub election_date: String,
    pub public_session: PublicSession,
    pub found_omissions: Vec<OmissionGroup>,
    pub recovered_omissions: Vec<OmissionGroup>,
    pub invalid_lists: Vec<OmissionGroup>,
    pub removed_candidates: Vec<RemovedCandidates>,
    pub removed_designations: Vec<RemovedDesignation>,
    pub corrected_designations: Vec<CorrectedDesignation>,
    pub valid_lists: Vec<DistrictLists>,
    pub numbered_based_on_votes: Vec<NumberedOnVotes>,
    pub numbered_based_on_districts: Vec<NumberedOnDistricts>,
    /// `None`: room to write during the session; empty: no objections raised.
    pub objections: Option<Vec<String>>,
    pub response_objections: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PublicSession {
    pub location: String,
    pub date: String,
    pub time: String,
    pub chair: String,
    pub members: Vec<String>,
}

/// Omissions for one list, identified by its designation and district(s).
#[derive(Debug, Deserialize)]
pub struct OmissionGroup {
    pub designation: String,
    pub electoral_district: String,
    pub omission_descriptions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RemovedCandidates {
    pub designation: String,
    pub electoral_district: String,
    pub candidates: Vec<RemovedCandidate>,
}

#[derive(Debug, Deserialize)]
pub struct RemovedCandidate {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RemovedDesignation {
    pub designation: String,
    pub electoral_district: String,
    pub first_candidate_name: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct CorrectedDesignation {
    pub first_candidate_name: String,
    pub electoral_district: String,
    pub submitted_designation: String,
    pub edited_designation: String,
}

#[derive(Debug, Deserialize)]
pub struct DistrictLists {
    pub electoral_district: String,
    pub lists: Vec<ValidList>,
}

#[derive(Debug, Deserialize)]
pub struct ValidList {
    pub designation: String,
    pub candidates: Vec<ValidListCandidate>,
}

#[derive(Debug, Deserialize)]
pub struct ValidListCandidate {
    pub last_name: String,
    pub initials: String,
    pub locality: String,
    pub position: usize,
}

#[derive(Debug, Deserialize)]
pub struct NumberedOnVotes {
    /// `None` when the number is still to be determined (rendered blank).
    pub position: Option<usize>,
    pub designation: String,
    pub previous_votes: u64,
}

#[derive(Debug, Deserialize)]
pub struct NumberedOnDistricts {
    /// `None` when the number is still to be determined (rendered blank).
    pub position: Option<usize>,
    pub designation: String,
    pub districts: u64,
}

impl Pdf for I4 {
    fn filename(&self) -> String {
        "i4-proces-verbaal.pdf".to_string()
    }

    fn document(&self) -> Textris {
        let mut doc = start_document(
            "Model I 4",
            "Proces-verbaal over geldigheid en nummering kandidatenlijsten",
            ModelLocale::Nl,
            None,
        );
        doc.paragraph(
            "Met dit formulier doet het centraal stembureau verslag van de zitting waarin is besloten over:",
        );
        doc.bullet_list([
            "de geldigheid en nummering van de kandidatenlijsten;",
            "het handhaven van de kandidaten op, en de aanduidingen bovenaan, de kandidatenlijsten.",
        ]);

        doc.h3_numbered("Verkiezing");
        doc.paragraph(text("Het gaat om de verkiezing van ").bold(&self.election_name));
        doc.paragraph(text("Dag van stemming ").bold(&self.election_date));

        doc.h3_numbered("Zitting");
        doc.paragraph(
            text("Het betreft de openbare zitting van het centraal stembureau in ")
                .bold(&self.public_session.location),
        );
        doc.paragraph(text("Datum zitting ").bold(&self.public_session.date));
        doc.paragraph(text("Tijdstip zitting ").bold(&self.public_session.time));

        doc.h3_numbered("Geconstateerde verzuimen");
        omissions(
            &mut doc,
            &self.found_omissions,
            "Bij het onderzoek naar de kandidatenlijsten waren geen herstelbare verzuimen geconstateerd.",
            "Bij het onderzoek naar de kandidatenlijsten waren de volgende herstelbare verzuimen geconstateerd:",
        );

        doc.h3_numbered("Herstelde verzuimen");
        omissions(
            &mut doc,
            &self.recovered_omissions,
            "Er zijn geen verzuimen hersteld.",
            "De volgende verzuimen zijn hersteld:",
        );

        doc.h3_numbered("Ongeldige lijsten");
        omissions(
            &mut doc,
            &self.invalid_lists,
            "Het centraal stembureau besluit dat geen lijst ongeldig is verklaard.",
            "Het centraal stembureau besluit dat de volgende lijsten ongeldig zijn verklaard:",
        );

        doc.h3_numbered("Geschrapte kandidaten");
        if self.removed_candidates.is_empty() {
            doc.paragraph(
                "Het centraal stembureau besluit dat geen kandidaat van een lijst is geschrapt.",
            );
        } else {
            doc.paragraph(
                "Het centraal stembureau besluit dat de volgende kandidaten van een lijst zijn geschrapt:",
            );
            doc.table_styled(
                &plain_table([Fraction(1), Fraction(1), Fraction(2)]),
                ["Aanduiding in de kieskring(en)", "naam kandidaat", "reden"],
                self.removed_candidates.iter().flat_map(|group| {
                    group.candidates.iter().enumerate().map(|(i, candidate)| {
                        [
                            text(group_label(
                                i,
                                &group.designation,
                                &group.electoral_district,
                            )),
                            text(&candidate.name),
                            text(&candidate.reason),
                        ]
                    })
                }),
            );
        }

        doc.h3_numbered("Geschrapte aanduidingen");
        if self.removed_designations.is_empty() {
            doc.paragraph(
                "Het centraal stembureau besluit dat geen aanduiding boven een lijst is geschrapt.",
            );
        } else {
            doc.paragraph(
                "Het centraal stembureau besluit dat de volgende aanduidingen boven een lijst zijn geschrapt:",
            );
            doc.table_styled(
                &plain_table([Fraction(1), Fraction(1), Fraction(2)]),
                [
                    "Aanduiding in de kieskring(en)",
                    "naam eerste kandidaat op de lijst",
                    "reden",
                ],
                self.removed_designations.iter().map(|removed| {
                    [
                        text(format!(
                            "{} in {}",
                            removed.designation, removed.electoral_district
                        )),
                        text(&removed.first_candidate_name),
                        text(&removed.reason),
                    ]
                }),
            );
        }

        doc.h3_numbered("Gecorrigeerde aanduiding");
        if self.corrected_designations.is_empty() {
            doc.paragraph(
                "Het centraal stembureau besluit dat geen aanduiding boven een lijst ambtshalve is aangepast.",
            );
        } else {
            doc.paragraph(
                "Het centraal stembureau besluit dat de volgende aanduidingen boven een lijst ambtshalve zijn aangepast:",
            );
            doc.table_styled(
                &plain_table([Fraction(1), Fraction(1), Fraction(2)]),
                [
                    "Naam eerste kandidaat in de kieskring(en)",
                    "vermelde aanduiding bij inlevering",
                    "aangepaste aanduiding",
                ],
                self.corrected_designations.iter().map(|corrected| {
                    [
                        text(format!(
                            "{} in {}",
                            corrected.first_candidate_name, corrected.electoral_district
                        )),
                        text(&corrected.submitted_designation),
                        text(&corrected.edited_designation),
                    ]
                }),
            );
        }

        doc.h3_numbered("Geldige lijsten");
        doc.paragraph(
            "Het centraal stembureau besluit dat de volgende lijsten geldig zijn verklaard:",
        );
        doc.page_break();
        for district in &self.valid_lists {
            doc.h4(format!("Kieskring {}", district.electoral_district));
            for (index, list) in district.lists.iter().enumerate() {
                doc.h5(format!("{}. {}", upper_alpha(index + 1), list.designation));
                doc.table_styled(
                    &column_table([Auto, Fraction(1), Fraction(1), Fraction(1)]),
                    ["", "naam kandidaat", "voorletters", "woonplaats"],
                    list.candidates.iter().map(|c| {
                        [
                            text(c.position.to_string()),
                            text(&c.last_name),
                            text(&c.initials),
                            text(&c.locality),
                        ]
                    }),
                );
                doc.page_break();
            }
        }

        doc.h3_numbered("Nummering van de kandidatenlijsten");
        doc.h4(
            "Nummering op grond van het aantal stemmen behaald bij de laatstgehouden verkiezing",
        );
        doc.paragraph(
            "Eerst zijn de kandidatenlijsten genummerd van de politieke groeperingen die een of meer zetels hebben behaald bij de laatstgehouden verkiezing, in de volgorde van de bij die verkiezing op de desbetreffende lijsten uitgebrachte aantallen stemmen. Voor zover nodig is rekening gehouden met samengevoegde aanduidingen. Bij een gelijk aantal stemmen is er genummerd via loting.",
        );
        doc.table_styled(
            &column_table_aligned(
                [Auto, Fraction(1), Auto],
                [Align::Left, Align::Left, Align::Right],
            ),
            [
                "nummer",
                "aanduiding politieke groepering",
                "aantal stemmen bij laatste verkiezing",
            ],
            self.numbered_based_on_votes.iter().map(|entry| {
                [
                    text(position_label(entry.position)),
                    text(&entry.designation),
                    text(entry.previous_votes.to_string()),
                ]
            }),
        );

        doc.h4("Nummering van de overige lijsten");
        doc.paragraph(
            "Vervolgens zijn de overige kandidatenlijsten genummerd in de volgorde van het aantal kieskringen waarvoor de lijst is ingeleverd. Bij een gelijk aantal kieskringen is er genummerd via loting.",
        );
        doc.table_styled(
            &column_table_aligned(
                [Auto, Fraction(1), Auto],
                [Align::Left, Align::Left, Align::Right],
            ),
            [
                "nummer",
                "aanduiding politieke groepering of naam eerste kandidaat",
                "aantal kieskringen waarvoor lijst geldt",
            ],
            self.numbered_based_on_districts.iter().map(|entry| {
                [
                    text(position_label(entry.position)),
                    text(&entry.designation),
                    text(entry.districts.to_string()),
                ]
            }),
        );

        doc.h3_numbered("Bezwaren van de aanwezige kiezers");
        match &self.objections {
            None => {
                doc.paragraph("Tijdens de zitting zijn");
                doc.task_list([
                    (false, "geen bezwaren ingebracht."),
                    (false, "de volgende bezwaren ingebracht:"),
                ]);
                // room for writing during the session
                doc.spacer(em(30.0));
            }
            Some(objections) if objections.is_empty() => {
                doc.paragraph("Tijdens de zitting zijn geen bezwaren ingebracht.");
            }
            Some(objections) => {
                doc.paragraph("Tijdens de zitting zijn de volgende bezwaren ingebracht:");
                doc.ordered_list_with(
                    ListMarker::LowerAlpha,
                    objections.iter().map(String::as_str),
                );
            }
        }
        if let Some(response) = &self.response_objections {
            doc.paragraph(response.as_str());
        }

        doc.h3_numbered("Ondertekening");
        let signing = TableStyle {
            header: false,
            striped: false,
            flush_first_column: true,
            columns: ColumnWidths::custom([Fraction(5), Fraction(4), Fraction(6)]),
            ..TableStyle::data()
        };
        doc.table_styled(
            &signing,
            [blank(), blank(), blank()],
            [[cell("Datum"), cell(&*self.public_session.date), blank()]],
        );
        let signing_tall = TableStyle {
            row_min_height: Some(em(3.5)),
            ..signing
        };
        let chair_row = [
            cell("Naam en handtekening voorzitter"),
            cell(&*self.public_session.chair),
            fill_in(),
        ];
        let member_rows = self
            .public_session
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                [
                    if index == 0 {
                        cell("Naam en handtekening leden")
                    } else {
                        blank()
                    },
                    cell(&**member),
                    fill_in(),
                ]
            });
        doc.table_styled(
            &signing_tall,
            [blank(), blank(), blank()],
            std::iter::once(chair_row).chain(member_rows),
        );

        doc
    }
}

/// A section listing omission groups: a paragraph when empty, otherwise an
/// intro and a two-column table with one row per omission, the first row of
/// each group labelled with its designation and district.
fn omissions(doc: &mut Textris, groups: &[OmissionGroup], none_text: &str, intro: &str) {
    if groups.is_empty() {
        doc.paragraph(none_text);
        return;
    }
    doc.paragraph(intro);
    doc.table_styled(
        &plain_table([Fraction(1), Fraction(2)]),
        ["Aanduiding in de kieskring(en)", "omschrijving verzuim"],
        groups.iter().flat_map(|group| {
            group
                .omission_descriptions
                .iter()
                .enumerate()
                .map(|(i, description)| {
                    [
                        text(group_label(
                            i,
                            &group.designation,
                            &group.electoral_district,
                        )),
                        text(description),
                    ]
                })
        }),
    );
}

/// The first-column label for grouped table rows: only the first row of a
/// group names the list.
fn group_label(index: usize, designation: &str, electoral_district: &str) -> String {
    if index == 0 {
        format!("{designation} in {electoral_district}")
    } else {
        String::new()
    }
}

/// A list number, blank when still to be determined.
fn position_label(position: Option<usize>) -> String {
    position.map(|p| p.to_string()).unwrap_or_default()
}

/// Uppercase letter numbering: 1 → `A`, 26 → `Z`, 27 → `AA`, …
fn upper_alpha(index: usize) -> String {
    let mut n = index;
    let mut out = Vec::new();
    while n > 0 {
        n -= 1;
        out.push(b'A' + (n % 26) as u8);
        n /= 26;
    }
    out.reverse();
    String::from_utf8(out).expect("ascii letters")
}
