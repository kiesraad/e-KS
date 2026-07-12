//! The official election PDF models, rendered in-process with
//! [`textris_pdf`].
//!
//! Each model lives in its own file (`h1`, `h3_1`, `h3_2`, `h4`, `h9`, `i4`),
//! mirroring the former Typst templates. [`layout`] holds the shared page
//! set-up and table styles, and [`inputs`] the shared input data types. Input
//! structs are deserializable so the example inputs in `src/models/example-inputs`
//! can be rendered directly (see `render_example` and the `pdf_diff` tool).

mod fonts;
pub mod h1;
pub mod h3;
mod h3_1;
mod h3_2;
pub mod h4;
pub mod h9;
pub mod i4;
pub mod inputs;
mod layout;

pub use fonts::fonts;

use serde::de::DeserializeOwned;
use textris_pdf::build::Textris;

use crate::AppError;

/// A document that renders to a PDF: it can build a [`Textris`] document and
/// knows its download file name.
pub trait Pdf: Sized {
    /// Build the document from the input data.
    fn document(&self) -> Textris;

    fn filename(&self) -> String;

    /// Render the PDF/A-2b bytes on a blocking thread (rendering is CPU-bound).
    // The trait is only consumed inside this crate, so auto trait bounds on
    // the returned future don't need to be nameable.
    #[allow(async_fn_in_trait)]
    async fn generate_bytes(&self) -> Result<Vec<u8>, AppError> {
        let document = self.document();
        Ok(
            tokio::task::spawn_blocking(move || document.render(fonts()))
                .await
                .map_err(|_| AppError::InternalServerError)??,
        )
    }
}

fn example<T: Pdf + DeserializeOwned>(input: serde_json::Value) -> Result<Vec<u8>, AppError> {
    let model: T =
        serde_json::from_value(input).map_err(|err| AppError::UserError(err.to_string()))?;
    Ok(model.document().render(fonts())?)
}

/// Render a model by template name from a JSON input, synchronously. Used by
/// the `pdf_diff` development tool to render the example inputs.
pub fn render_example(template: &str, input: serde_json::Value) -> Result<Vec<u8>, AppError> {
    match template {
        "model-h1" => example::<h1::H1>(input),
        "model-h3-1" | "model-h3-2" => example::<h3::H3>(input),
        "model-h4" => example::<h4::H4>(input),
        "model-h9" => example::<h9::H9>(input),
        "model-i4" => example::<i4::I4>(input),
        _ => Err(AppError::NotFound(format!("unknown model: {template}"))),
    }
}

/// The template names accepted by [`render_example`].
pub const TEMPLATES: &[&str] = &[
    "model-h1",
    "model-h3-1",
    "model-h3-2",
    "model-h4",
    "model-h9",
    "model-i4",
];
