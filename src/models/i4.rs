//! Model I 4: Proces-verbaal over geldigheid en nummering kandidatenlijsten.
//! This model is Dutch-only.

use textris_pdf::{
    build::{Text, Textris, blank, cell, fill_in, text},
    model::ListMarker,
    theme::{
        Align, ColumnWidth,
        ColumnWidth::{Auto, Fraction},
        ColumnWidths, TableStyle, em,
    },
};

use super::{
    Pdf,
    layout::{column_table, start_document},
};
use crate::core::ModelLocale;

/// The unstriped three-column table used by the omission and decision sections.
fn plain_table(widths: impl IntoIterator<Item = ColumnWidth>) -> TableStyle {
    TableStyle {
        striped: false,
        ..column_table(widths)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct PublicSession {
    pub location: String,
    pub date: String,
    pub time: String,
    pub chair: String,
    pub members: Vec<String>,
}

/// Omissions for one list, identified by its designation and district(s).
#[derive(Debug)]
pub struct OmissionGroup {
    pub designation: String,
    pub electoral_district: String,
    pub omission_descriptions: Vec<String>,
}

#[derive(Debug)]
pub struct RemovedCandidates {
    pub designation: String,
    pub electoral_district: String,
    pub candidates: Vec<RemovedCandidate>,
}

#[derive(Debug)]
pub struct RemovedCandidate {
    pub name: String,
    pub reason: String,
}

#[derive(Debug)]
pub struct RemovedDesignation {
    pub designation: String,
    pub electoral_district: String,
    pub first_candidate_name: String,
    pub reason: String,
}

#[derive(Debug)]
pub struct CorrectedDesignation {
    pub first_candidate_name: String,
    pub electoral_district: String,
    pub submitted_designation: String,
    pub edited_designation: String,
}

#[derive(Debug)]
pub struct DistrictLists {
    pub electoral_district: String,
    pub lists: Vec<ValidList>,
}

#[derive(Debug)]
pub struct ValidList {
    pub designation: String,
    pub candidates: Vec<ValidListCandidate>,
}

#[derive(Debug)]
pub struct ValidListCandidate {
    pub last_name: String,
    pub initials: String,
    pub locality: String,
    pub position: usize,
}

#[derive(Debug)]
pub struct NumberedOnVotes {
    /// `None` when the number is still to be determined (rendered blank).
    pub position: Option<usize>,
    pub designation: String,
    pub previous_votes: u64,
}

#[derive(Debug)]
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
        optional_table(
            &mut doc,
            self.removed_candidates.is_empty(),
            "Het centraal stembureau besluit dat geen kandidaat van een lijst is geschrapt.",
            "Het centraal stembureau besluit dat de volgende kandidaten van een lijst zijn geschrapt:",
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

        doc.h3_numbered("Geschrapte aanduidingen");
        optional_table(
            &mut doc,
            self.removed_designations.is_empty(),
            "Het centraal stembureau besluit dat geen aanduiding boven een lijst is geschrapt.",
            "Het centraal stembureau besluit dat de volgende aanduidingen boven een lijst zijn geschrapt:",
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

        doc.h3_numbered("Gecorrigeerde aanduiding");
        optional_table(
            &mut doc,
            self.corrected_designations.is_empty(),
            "Het centraal stembureau besluit dat geen aanduiding boven een lijst ambtshalve is aangepast.",
            "Het centraal stembureau besluit dat de volgende aanduidingen boven een lijst ambtshalve zijn aangepast:",
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

        doc.h3_numbered("Geldige lijsten");
        valid_lists_section(&mut doc, &self.valid_lists);

        doc.h3_numbered("Nummering van de kandidatenlijsten");
        doc.h4(
            "Nummering op grond van het aantal stemmen behaald bij de laatstgehouden verkiezing",
        );
        doc.paragraph(
            "Eerst zijn de kandidatenlijsten genummerd van de politieke groeperingen die een of meer zetels hebben behaald bij de laatstgehouden verkiezing, in de volgorde van de bij die verkiezing op de desbetreffende lijsten uitgebrachte aantallen stemmen. Voor zover nodig is rekening gehouden met samengevoegde aanduidingen. Bij een gelijk aantal stemmen is er genummerd via loting.",
        );
        numbering_table(
            &mut doc,
            [
                "nummer",
                "aanduiding politieke groepering",
                "aantal stemmen bij laatste verkiezing",
            ],
            self.numbered_based_on_votes.iter().map(|entry| {
                (
                    entry.position,
                    entry.designation.as_str(),
                    entry.previous_votes,
                )
            }),
        );

        doc.h4("Nummering van de overige lijsten");
        doc.paragraph(
            "Vervolgens zijn de overige kandidatenlijsten genummerd in de volgorde van het aantal kieskringen waarvoor de lijst is ingeleverd. Bij een gelijk aantal kieskringen is er genummerd via loting.",
        );
        numbering_table(
            &mut doc,
            [
                "nummer",
                "aanduiding politieke groepering of naam eerste kandidaat",
                "aantal kieskringen waarvoor lijst geldt",
            ],
            self.numbered_based_on_districts
                .iter()
                .map(|entry| (entry.position, entry.designation.as_str(), entry.districts)),
        );

        doc.h3_numbered("Bezwaren van de aanwezige kiezers");
        objections_section(&mut doc, &self.objections, &self.response_objections);

        doc.h3_numbered("Ondertekening");
        signing_section(&mut doc, &self.public_session);

        doc
    }
}

/// A section that is either a single "none" paragraph, or an intro line plus
/// the standard three-column table. Shared by the "geschrapte kandidaten",
/// "geschrapte aanduidingen" and "gecorrigeerde aanduiding" sections.
fn optional_table(
    doc: &mut Textris,
    is_empty: bool,
    none_text: &str,
    intro: &str,
    header: [&str; 3],
    rows: impl IntoIterator<Item = [Text; 3]>,
) {
    if is_empty {
        doc.paragraph(none_text);
        return;
    }
    doc.paragraph(intro);
    doc.table_styled(
        &plain_table([Fraction(1), Fraction(1), Fraction(2)]),
        header,
        rows,
    );
}

/// One of the two "Nummering" tables: number, designation and a right-aligned
/// count. `entries` yields `(position, designation, count)` per row.
fn numbering_table<'a>(
    doc: &mut Textris,
    header: [&str; 3],
    entries: impl IntoIterator<Item = (Option<usize>, &'a str, u64)>,
) {
    doc.table_styled(
        &TableStyle {
            align: vec![Align::Left, Align::Left, Align::Right],
            ..column_table([Auto, Fraction(1), Auto])
        },
        header,
        entries.into_iter().map(|(position, designation, count)| {
            [
                text(position_label(position)),
                text(designation),
                text(count.to_string()),
            ]
        }),
    );
}

/// The "Geldige lijsten" body: each district on its own, with a candidate table
/// per valid list and a page break between lists.
fn valid_lists_section(doc: &mut Textris, districts: &[DistrictLists]) {
    doc.paragraph("Het centraal stembureau besluit dat de volgende lijsten geldig zijn verklaard:");

    for district in districts {
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
}

/// The "Bezwaren" body: the objections raised (or write-in space when the
/// session is still open) and any recorded response.
fn objections_section(
    doc: &mut Textris,
    objections: &Option<Vec<String>>,
    response: &Option<String>,
) {
    match objections {
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
    if let Some(response) = response {
        doc.paragraph(response.as_str());
    }
}

/// The "Ondertekening" body: the date row and the tall signature rows for the
/// chair and the members.
fn signing_section(doc: &mut Textris, session: &PublicSession) {
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
        [[cell("Datum"), cell(&*session.date), blank()]],
    );
    let signing_tall = TableStyle {
        row_min_height: Some(em(3.5)),
        ..signing
    };
    let chair_row = [
        cell("Naam en handtekening voorzitter"),
        cell(&*session.chair),
        fill_in(),
    ];
    let member_rows = session.members.iter().enumerate().map(|(index, member)| {
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
}

/// A section listing omission groups: a paragraph when empty, otherwise an
/// intro and a two-column table with one row per group, its designation and
/// district next to all of the group's omissions stacked in one cell.
fn omissions(doc: &mut Textris, groups: &[OmissionGroup], none_text: &str, intro: &str) {
    if groups.is_empty() {
        doc.paragraph(none_text);
        return;
    }
    doc.paragraph(intro);
    doc.table_styled(
        &plain_table([Fraction(1), Fraction(2)]),
        ["Aanduiding in de kieskring(en)", "omschrijving verzuim"],
        groups.iter().map(|group| {
            [
                text(format!(
                    "{} in {}",
                    group.designation, group.electoral_district
                )),
                stacked(group.omission_descriptions.iter().map(String::as_str)),
            ]
        }),
    );
}

/// Combine several values into one cell, each on its own line. Mirrors a Typst
/// `table.cell(rowspan: …)` group (which textris-pdf cannot express): the label
/// column names the group once next to its stacked entries. Used where a single
/// data column accompanies the label; tables that pair multiple data columns
/// per entry keep one row per entry (see [`group_label`]) so the columns stay
/// aligned.
fn stacked<'a>(lines: impl IntoIterator<Item = &'a str>) -> Text {
    let mut cell = Text::new();
    for (index, line) in lines.into_iter().enumerate() {
        if index > 0 {
            cell = cell.line_break();
        }
        cell = cell.normal(line);
    }
    cell
}

/// The first-column label for grouped table rows: only the first row of a
/// group names the list, mirroring a Typst `table.cell(rowspan: …)` label.
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
