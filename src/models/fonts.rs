//! The typefaces used by the PDF models, embedded in the binary.
//!
//! DM Sans (roman + italic) and Geist Mono variable fonts; the DM Sans files
//! are patched to cover the full Teletex character set (see
//! `src/models/fonts/DM_Sans/modifications.md`).

use std::sync::OnceLock;

use textris_pdf::fonts::Fonts;

static DM_SANS: &[u8] = include_bytes!("fonts/DM_Sans/DMSans-Variable.ttf");
static DM_SANS_ITALIC: &[u8] = include_bytes!("fonts/DM_Sans/DMSans-Italic-Variable.ttf");
static GEIST_MONO: &[u8] = include_bytes!("fonts/Geist_Mono/GeistMono-Variable.ttf");

/// The shared, lazily-initialized font set (parsing fonts is done once per
/// process).
pub fn fonts() -> &'static Fonts {
    static FONTS: OnceLock<Fonts> = OnceLock::new();
    FONTS.get_or_init(|| {
        Fonts::from_variable(DM_SANS, DM_SANS_ITALIC, GEIST_MONO)
            .expect("embedded fonts should parse")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use textris_pdf::fonts::Style;

    #[test]
    fn fonts_load_and_cover_the_teletex_additions() {
        let fonts = fonts();
        // Widths must be positive for the glyphs patched into DM Sans.
        for style in [
            Style::Regular,
            Style::Bold,
            Style::Italic,
            Style::BoldItalic,
        ] {
            let width = fonts.measure(style, "ĈĉĜĝĤĥĴĵĸŉŜŝŦŧß", 9.0);
            assert!(width > 0.0);
        }
    }
}
