//! Translation macro definitions backed by generated locale strings.
//! Used in rust sources with the `trans!` macro, and in templates with the `|trans` filter.
include!(concat!(env!("OUT_DIR"), "/locales.rs"));

/// Translate a key for the given locale and format with optional arguments.
#[macro_export]
macro_rules! trans {
    ($key:tt, $locale:expr $(, $args:expr)* $(,)?) => {{
        match $locale {
            $crate::Locale::En => format!($crate::translate::t_en!($key) $(, $args)*),
            $crate::Locale::Nl => format!($crate::translate::t_nl!($key) $(, $args)*),
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::translate::{LOCALE_EN, LOCALE_NL};

    include!("../../locales/utils/find_used_keys.rs");

    #[test]
    fn test_unused_translation_keys() {
        let used_keys = find_used_keys(std::path::Path::new("./"));

        for key in LOCALE_NL.keys() {
            assert!(
                used_keys.contains(&key.to_string()),
                "Translation key '{key}' (in locales/nl) is not used in any template or source file"
            );
        }

        for key in LOCALE_EN.keys() {
            assert!(
                used_keys.contains(&key.to_string()),
                "Translation key '{key}' (in locales/en) is not used in any template or source file"
            );
        }

        for key in used_keys {
            assert!(
                LOCALE_NL.contains_key(&key),
                "Translation key '{key}' is used in a template or source file, but missing in locales/nl"
            );

            assert!(
                LOCALE_EN.contains_key(&key),
                "Translation key '{key}' is used in a template or source file, but missing in locales/en"
            );
        }
    }
}
