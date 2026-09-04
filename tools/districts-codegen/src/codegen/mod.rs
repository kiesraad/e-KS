//! Renders the `Districts` model into the three `*_generated.rs` files
//! included by the `eks` crate, one enum per file.
mod districts;
mod provinces;
mod water_councils;

use std::path::Path;

use proc_macro2::TokenStream;

pub(crate) use districts::write_districts_file;
pub(crate) use provinces::write_provinces_file;
pub(crate) use water_councils::write_water_councils_file;

fn write_file(tokens: TokenStream, filename: &'static str, out_dir: &Path) {
    let file = syn::parse2(tokens).expect("Failed to parse generated tokens");
    let path = out_dir.join(filename);
    std::fs::write(&path, prettyplease::unparse(&file)).expect("Failed to write generated file");
}
