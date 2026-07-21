The variable fonts `DMSans-Variable.ttf` and `DMSans-Italic-Variable.ttf` are
based on the DM Sans variable fonts from [Google Fonts](https://github.com/google/fonts/tree/main/ofl/dmsans),
modified to include all glyphs of the Teletex set (allowed in candidate names
by the EML_NL standard but not fully covered by DM Sans).

The added glyphs mirror the FontForge additions previously made to the static
`DMSans-Regular.ttf` (see git history):

- Ĉ ĉ Ĝ ĝ Ĥ ĥ Ĵ ĵ Ŝ ŝ are composites of the base letter and U+0302, so they
  follow the weight axis.
- ŉ is a composite of quoteright + n.
- ĸ Ŧ ŧ and U+FFFD are outline copies from the modified static font (they do
  not vary with weight).
- U+00AD (soft hyphen) is an empty zero-width glyph.
