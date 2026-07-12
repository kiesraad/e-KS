//! Shared page set-up and styles for the PDF models, mirroring the former
//! `models/layout.typ`.

use textris_pdf::{
    Color,
    build::{Text, Textris, blank, cell, fill_in, mono, text},
    model::SectionContent,
    theme::{Align, BoxStyle, ColumnWidth, ColumnWidths, TableStyle, Theme, em},
};

use super::inputs::Date;
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
