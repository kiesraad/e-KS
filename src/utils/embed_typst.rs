//! Embedded Typst renderer: builds and caches a [`PdfContext`] from assets
//! baked into the binary at compile time.
use std::sync::{Arc, OnceLock};

use typst_webservice::PdfContext;

const TYPST_FILES: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/typst_files.rs"));

static CONTEXT: OnceLock<Arc<PdfContext>> = OnceLock::new();

/// Returns a shared, lazily-initialized [`PdfContext`] built from the embedded
/// Typst assets. The context is cloned cheaply on subsequent calls.
pub fn pdf_context() -> Arc<PdfContext> {
    CONTEXT
        .get_or_init(|| {
            Arc::new(
                PdfContext::from_assets(TYPST_FILES).expect("failed to load embedded Typst assets"),
            )
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_context_initializes_and_is_cached() {
        let first = pdf_context();
        let second = pdf_context();
        assert!(Arc::ptr_eq(&first, &second));
    }
}
