//! Shared page set-up and styles for the PDF models.

use textris_pdf::{
    build::{Text, Textris, blank, cell, fill_in, text},
    model::SectionContent,
    theme::{BoxStyle, ColumnWidth, ColumnWidths, TableStyle, Theme, em},
};

use super::inputs::{Candidate, ElectoralDistricts, ModelData};
use crate::core::ModelLocale;

/// Pick the Dutch or the Frisian variant of a text. Keeps both languages next
/// to each other at the call sites.
pub(super) fn translator(
    locale: ModelLocale,
) -> impl Fn(&'static str, &'static str) -> &'static str {
    move |dutch, frisian| match locale {
        ModelLocale::Nl => dutch,
        ModelLocale::Fry => frisian,
    }
}

fn theme() -> Theme {
    let mut theme = Theme::default();
    theme.font_size.h5 = em(1.0);
    theme.table.inset_y = em(0.2);
    theme.spacing.heading_above.h3 = em(1.5);
    theme.spacing.heading_above.h4 = em(1.5);
    theme.spacing.heading_above.h5 = em(1.5);
    // Header baseline 30% of the top margin above the content edge, matching
    theme.page.header_offset = 0.3 * theme.page.margin_y;
    theme
}

/// Start a model document: theme, header (`<model> - <name>`), footer
/// (version/hash and page counter) and the title block. All H-models carry
/// the event version and hash in their footer; I 4 does not.
pub(super) fn start_document(
    model: &str,
    name: &str,
    locale: ModelLocale,
    version: Option<(usize, &str)>,
) -> Textris {
    let mut doc = Textris::with_theme(theme());
    let trans = translator(locale);

    // Metadata for the accessible (PDF/UA) output: a document title and its
    // primary language as a BCP 47 tag (Dutch, or West Frisian for the Frisian
    // variants).
    doc.title(format!("{model} - {name}"));
    doc.language(match locale {
        ModelLocale::Nl => "nl",
        ModelLocale::Fry => "fy",
    });

    doc.header_right(format!("{model} - {name}"));

    if let Some((event_id, sha_hash)) = version {
        doc.footer_left(
            Text::new()
                .muted(trans("Versie:", "Ferzje:"))
                .normal(" ")
                .mono(event_id.to_string())
                .normal("   ")
                .muted("Hash:")
                .normal(" ")
                .mono(sha_hash),
        );
    }

    let (page, of) = match locale {
        ModelLocale::Nl => ("Pagina", "van"),
        ModelLocale::Fry => ("Side", "fan"),
    };
    doc.footer_right(SectionContent::page_counter(move |current, total| {
        text(format!("{page} {current} {of} {total}"))
    }));

    doc.h2(model);
    doc.h1(name);
    doc
}

/// The opening shared by the H models: [`start_document`] with the event
/// version, the intro paragraph, an optional warning box and the numbered
/// "Verkiezing" section.
pub(super) fn start_h_document(
    common: &ModelData,
    model: &str,
    name: &str,
    intro: &str,
    warning_box: Option<(&str, &str)>,
    election_intro: &str,
) -> Textris {
    let trans = translator(common.locale);
    let mut doc = start_document(
        model,
        name,
        common.locale,
        Some((common.event_id, &common.sha_hash)),
    );
    doc.paragraph(intro);
    if let Some((title, body)) = warning_box {
        warning(&mut doc, title, body);
    }
    bold_value_section(
        &mut doc,
        trans("Verkiezing", "Ferkiezing"),
        election_intro,
        &common.election_name,
    );
    doc
}

/// A numbered section whose single paragraph ends in a bold value, like the
/// "Verkiezing" and designation sections that open the models.
pub(super) fn bold_value_section(doc: &mut Textris, heading: &str, intro: &str, value: &str) {
    doc.h3_numbered(heading);
    doc.paragraph(text(intro).bold(value));
}

/// The numbered "Kieskringen" section; single-district elections omit it.
/// `all` is the bold phrase when the choice covers all districts, `some`
/// optionally bolds a lead-in above the listed district names.
pub(super) fn districts_section(
    doc: &mut Textris,
    locale: ModelLocale,
    districts: &ElectoralDistricts,
    intro: &str,
    all: &str,
    some: Option<&str>,
) {
    if *districts == ElectoralDistricts::OnlyOne {
        return;
    }
    let trans = translator(locale);
    doc.h3_numbered(trans("Kieskringen", "Kiesrûnten"));
    match districts {
        ElectoralDistricts::All => {
            doc.paragraph(text(intro).bold(all));
        }
        ElectoralDistricts::Some(names) => {
            match some {
                Some(lead) => doc.paragraph(text(intro).bold(lead)),
                None => doc.paragraph(text(intro)),
            };
            doc.paragraph(names.join(", "));
        }
        ElectoralDistricts::OnlyOne => {}
    }
}

/// The numbered "Kandidaten op de lijst" section: the heading and the standard
/// four-column table (nummer, naam, voorletters, woonplaats). Shared by H 3,
/// H 4 and H 9.
pub(super) fn candidates_section(doc: &mut Textris, locale: ModelLocale, candidates: &[Candidate]) {
    let trans = translator(locale);
    doc.h3_numbered(trans("Kandidaten op de lijst", "Kandidaten op de list"));
    doc.table_styled(
        &column_table([
            ColumnWidth::Auto,
            ColumnWidth::Fraction(1),
            ColumnWidth::Fraction(1),
            ColumnWidth::Fraction(1),
        ]),
        [
            "",
            trans("naam", "namme"),
            trans("voorletters", "foarletters"),
            trans("woonplaats", "wenplak"),
        ],
        candidates.iter().map(|c| {
            [
                text(c.position.to_string()),
                text(&c.last_name),
                text(&c.initials),
                text(&c.locality),
            ]
        }),
    );
}

/// A highlighted warning box below the title block, with a bold first line.
/// The background matches the zebra-stripe colour of the tables.
pub(super) fn warning(doc: &mut Textris, title: &str, body: &str) {
    let style = BoxStyle {
        background: doc.theme().palette.highlight,
        ..BoxStyle::callout()
    };
    doc.boxed_styled(&style, |boxed| {
        boxed.paragraph(Text::new().bold(title).line_break().normal(body));
    });
}

/// A data table with the given column widths: striped rows, an italic header.
pub(super) fn column_table(widths: impl IntoIterator<Item = ColumnWidth>) -> TableStyle {
    TableStyle {
        columns: ColumnWidths::custom(widths),
        ..TableStyle::data()
    }
}

/// A label-table row with a tall fill-in area for a hand-written signature.
pub(super) fn signature_line(doc: &mut Textris, label: &str) {
    let style = TableStyle {
        row_min_height: Some(em(3.5)),
        ..TableStyle::label()
    };
    doc.table_styled(&style, [blank(), blank()], [[cell(label), fill_in()]]);
}
