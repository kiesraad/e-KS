//! Shared page set-up and styles for the PDF models, mirroring the former
//! `models/layout.typ`.

use textris_pdf::{
    Color,
    build::{Text, Textris, blank, cell, fill_in, mono, text},
    model::SectionContent,
    theme::{Align, BoxStyle, ColumnWidth, ColumnWidths, TableStyle, Theme, em},
};

use super::inputs::{Candidate, Date, ModelData};
use crate::core::ModelLocale;

/// Background colour for warning boxes and zebra-striped table rows.
pub(super) fn highlight_colour() -> Color {
    Color::new(0xF6, 0xF6, 0xF6)
}

/// Pick the Dutch or the Frisian variant of a text, like `translator` in the
/// former `layout.typ`. Keeps both languages next to each other at the call
/// sites.
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
    // Typst's `===` headings render at the body size.
    theme.font_size.h5 = em(1.0);
    // Tighter rows, close to the Typst tables (`rows: 1.45em`).
    theme.table.inset_y = em(0.2);
    // Tighter section spacing so documents paginate like the Typst originals.
    theme.spacing.heading_above.h3 = em(1.5);
    theme.spacing.heading_above.h4 = em(1.5);
    theme.spacing.heading_above.h5 = em(1.5);
    theme
}

/// Start a model document: theme, header (`<model> - <name>`), footer
/// (version/hash and page counter) and the title block.
pub(super) fn start_document(
    model: &str,
    name: &str,
    locale: ModelLocale,
    version: Option<(usize, &str)>,
) -> Textris {
    let mut doc = Textris::with_theme(theme());
    let trans = translator(locale);

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

/// Start a model document that carries the event version and hash in its
/// footer. All H-models are versioned this way (I 4 is not).
pub(super) fn start_versioned(model: &str, name: &str, common: &ModelData) -> Textris {
    start_document(
        model,
        name,
        common.locale,
        Some((common.event_id, &common.sha_hash)),
    )
}

/// The numbered "Verkiezing" section: the heading and an intro line ending in
/// the bold election name. Shared by H 1, H 3, H 4 and H 9.
pub(super) fn election_section(
    doc: &mut Textris,
    locale: ModelLocale,
    intro_nl: &'static str,
    intro_fry: &'static str,
    election_name: &str,
) {
    let trans = translator(locale);
    doc.h3_numbered(trans("Verkiezing", "Ferkiezing"));
    doc.paragraph(text(trans(intro_nl, intro_fry)).bold(election_name));
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
pub(super) fn warning(doc: &mut Textris, title: &str, body: &str) {
    let style = BoxStyle {
        background: highlight_colour(),
        ..BoxStyle::callout()
    };
    doc.boxed_styled(&style, |boxed| {
        boxed.paragraph(Text::new().bold(title).line_break().normal(body));
    });
}

/// A standard "Let op!" warning callout with the given body. Shared by H 4 and
/// H 9.
pub(super) fn warning_let_op(
    doc: &mut Textris,
    locale: ModelLocale,
    body_nl: &'static str,
    body_fry: &'static str,
) {
    let trans = translator(locale);
    warning(
        doc,
        trans("Let op!", "Tink der om!"),
        trans(body_nl, body_fry),
    );
}

/// `column_table` from `layout.typ`: striped rows and an italic header.
pub(super) fn column_table(widths: impl IntoIterator<Item = ColumnWidth>) -> TableStyle {
    TableStyle {
        columns: ColumnWidths::custom(widths),
        ..TableStyle::data()
    }
}

/// `column_table` with per-column alignment.
pub(super) fn column_table_aligned(
    widths: impl IntoIterator<Item = ColumnWidth>,
    align: impl IntoIterator<Item = Align>,
) -> TableStyle {
    TableStyle {
        align: align.into_iter().collect(),
        ..column_table(widths)
    }
}

/// `plain_table` from `layout.typ`: an italic header, no stripes.
pub(super) fn plain_table(widths: impl IntoIterator<Item = ColumnWidth>) -> TableStyle {
    TableStyle {
        striped: false,
        ..column_table(widths)
    }
}

/// A label-table row with a tall fill-in area for a hand-written signature
/// (`fill_in(height: 4em)` in the old `layout.typ`).
pub(super) fn signature_line(doc: &mut Textris, label: &str) {
    let style = TableStyle {
        row_min_height: Some(em(3.5)),
        ..TableStyle::label()
    };
    doc.table_styled(&style, [blank(), blank()], [[cell(label), fill_in()]]);
}

/// A date in `dd-mm-yyyy`, in the monospaced font.
pub(super) fn date(date: &Date) -> Text {
    mono(format!(
        "{:02}-{:02}-{:04}",
        date.day, date.month, date.year
    ))
}
