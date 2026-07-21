//! Askama wiring for the Markdown dialect templates the PDF models are
//! written in (`src/models/templates/`, parsed by
//! [`textris_pdf::markdown::parse`]).
//!
//! Interpolated data must never be able to change the document structure, so
//! every `{{ }}` value passes through the crate's escape functions:
//! [`MarkdownEscaper`] auto-escapes flow contexts (registered for the `.md`
//! extension in `askama.toml`), and [`filters`] cover the table-cell and
//! verbatim contexts.

use askama::filters::Safe;
use chrono::NaiveDate;
use textris_pdf::markdown;

use crate::core::constants::DEFAULT_DATE_FORMAT;

/// The `.md` auto-escaper: [`markdown::escape`] for flow contexts
/// (paragraphs, headings, list items, quotes).
#[derive(Debug, Clone, Copy)]
pub struct MarkdownEscaper;

impl askama::filters::Escaper for MarkdownEscaper {
    fn write_escaped_str<W: std::fmt::Write>(&self, mut dest: W, string: &str) -> std::fmt::Result {
        dest.write_str(&markdown::escape(string))
    }
}

/// Filters for the escaping contexts the flow auto-escaper does not cover,
/// plus the shared `display` filter for optional values.
pub mod filters {
    use super::*;

    pub use crate::filters::display;

    /// Escape a value interpolated into a table cell.
    #[askama::filter_fn]
    pub fn cell<T: std::fmt::Display>(
        value: T,
        _: &dyn askama::Values,
    ) -> askama::Result<Safe<String>> {
        Ok(Safe(markdown::escape_cell(&value.to_string())))
    }

    /// Wrap a value in a verbatim (mono) span inside a table cell.
    #[askama::filter_fn]
    pub fn mono_cell<T: std::fmt::Display>(
        value: T,
        _: &dyn askama::Values,
    ) -> askama::Result<Safe<String>> {
        Ok(Safe(markdown::mono_cell(&value.to_string())))
    }

    /// Format a date in the model date format (`31-12-2027`).
    #[askama::filter_fn]
    pub fn date(value: &NaiveDate, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(value.format(DEFAULT_DATE_FORMAT).to_string())
    }

    /// Uppercase letter numbering for list labels: 1 → `A`, 26 → `Z`,
    /// 27 → `AA`, …
    #[askama::filter_fn]
    pub fn upper_alpha(index: &usize, _: &dyn askama::Values) -> askama::Result<String> {
        let mut n = *index;
        let mut out = Vec::new();
        while n > 0 {
            n -= 1;
            out.push(b'A' + (n % 26) as u8);
            n /= 26;
        }
        out.reverse();
        Ok(String::from_utf8(out).expect("ascii letters"))
    }
}

/// Bind a PDF model to one Markdown template (one per locale and variant): an
/// askama wrapper that derefs to the model, so the template reads its fields
/// and methods directly.
macro_rules! model_template {
    ($wrapper:ident, $model:ident, $path:literal) => {
        #[derive(askama::Template)]
        #[template(path = $path)]
        struct $wrapper<'a>(&'a $model);

        impl std::ops::Deref for $wrapper<'_> {
            type Target = $model;

            fn deref(&self) -> &Self::Target {
                self.0
            }
        }
    };
}
pub(super) use model_template;

#[cfg(test)]
mod tests {
    use super::filters;

    fn upper_alpha(index: usize) -> String {
        filters::upper_alpha::default()
            .execute(&index, askama::NO_VALUES)
            .unwrap()
    }

    #[test]
    fn upper_alpha_single_letters() {
        assert_eq!(upper_alpha(1), "A");
        assert_eq!(upper_alpha(2), "B");
        assert_eq!(upper_alpha(26), "Z");
    }

    #[test]
    fn upper_alpha_double_letters() {
        assert_eq!(upper_alpha(27), "AA");
        assert_eq!(upper_alpha(28), "AB");
        assert_eq!(upper_alpha(52), "AZ");
        assert_eq!(upper_alpha(53), "BA");
        assert_eq!(upper_alpha(702), "ZZ");
    }

    #[test]
    fn upper_alpha_triple_letters() {
        assert_eq!(upper_alpha(703), "AAA");
        assert_eq!(upper_alpha(704), "AAB");
    }

    #[test]
    fn upper_alpha_zero_is_empty() {
        assert_eq!(upper_alpha(0), "");
    }
}
