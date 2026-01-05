//! Translation macro definitions backed by generated locale strings.
//! Used in templates and handlers via the `t!` macro.
//! Passing a locale yields the translated string.
//! Passing only a key yields an array of all translations.

include!(concat!(env!("OUT_DIR"), "/locales.rs"));

#[macro_export]
macro_rules! t {
    ($key:tt) => {{
        &[$crate::translate::t_en!($key), $crate::translate::t_nl!($key)]
    }};
    ($key:tt, $locale:expr $(, $args:expr)* $(,)?) => {{
        match $locale {
            $crate::locale::Locale::En => format!($crate::translate::t_en!($key) $(, $args)*),
            $crate::locale::Locale::Nl => format!($crate::translate::t_nl!($key) $(, $args)*),
        }
    }};
}
