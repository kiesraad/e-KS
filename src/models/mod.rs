//! The official election PDF models, rendered in-process with
//! [`textris_pdf`].
//!
//! Each model lives in its own file (`h1`, `h3`, `h4`, `h9`, `i4`); H 3 covers
//! both the H 3-1 and H 3-2 variants. [`layout`] holds the shared page
//! set-up and table styles, and [`inputs`] the shared input data types. Input
//! structs are deserializable so the example inputs in `src/models/example-inputs`
//! can be rendered directly (see `render_example` and the `pdf_diff` tool).

mod fonts;
pub mod h1;
pub mod h3;
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

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use serde_json::{Value, json};

    use super::*;

    fn examples_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/models/example-inputs")
    }

    fn read_example(name: &str) -> Value {
        let path = examples_dir().join(name);
        let json = fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {name}: {err}"));
        serde_json::from_str(&json).unwrap_or_else(|err| panic!("parse {name}: {err}"))
    }

    #[track_caller]
    fn assert_pdf(bytes: &[u8], ctx: &str) {
        assert!(bytes.starts_with(b"%PDF"), "{ctx}: output is not a PDF");
        assert!(
            bytes.len() > 1000,
            "{ctx}: PDF unexpectedly small ({} bytes)",
            bytes.len()
        );
    }

    #[track_caller]
    fn render(template: &str, input: Value) -> Vec<u8> {
        render_example(template, input).unwrap_or_else(|err| panic!("render {template}: {err:?}"))
    }

    /// Every JSON example input renders to a valid PDF. This drives all six
    /// document builders (`h1`, `h3-1`, `h3-2`, `h4`, `h9`, `i4`) together with
    /// the shared layout and input-parsing code, end to end.
    #[test]
    fn renders_every_example_input() {
        let mut rendered = 0;
        for entry in fs::read_dir(examples_dir()).expect("read example-inputs dir") {
            let path = entry.unwrap().path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let template = stem.rsplit_once("-example-").map_or(stem, |(name, _)| name);
            assert_pdf(
                &render(template, read_example(&format!("{stem}.json"))),
                stem,
            );
            rendered += 1;
        }
        assert_eq!(rendered, 17, "expected to render every example input");
    }

    #[test]
    fn unknown_template_is_not_found() {
        assert!(matches!(
            render_example("model-h7", json!({})),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn malformed_input_is_a_user_error() {
        // A model needs many fields; an empty object cannot deserialise.
        for template in TEMPLATES {
            assert!(
                matches!(
                    render_example(template, json!({})),
                    Err(AppError::UserError(_))
                ),
                "{template} should reject an empty object with a user error",
            );
        }
    }

    /// H 1's attachment checklist branches on the election type; render each so
    /// every branch (including EP and the non-resident electoral college) runs.
    #[test]
    fn h1_attachments_cover_every_election_type() {
        for election_type in ["TK", "EK", "GR", "PS", "WS", "EP", "KC", "KCNI", "ER"] {
            let mut input = read_example("model-h1-example-2.json");
            input["election_type"] = json!(election_type);
            assert_pdf(&render("model-h1", input), election_type);
        }
    }

    /// H 4 renders the mayor's statement for every election type except the
    /// senate (EK), with wording that depends on who keeps the voter register.
    #[test]
    fn h4_mayor_section_per_election_type() {
        for election_type in ["EK", "TK", "GR", "ER"] {
            let mut input = read_example("model-h4-example-1.json");
            input["election_type"] = json!(election_type);
            assert_pdf(&render("model-h4", input), election_type);
        }
    }

    /// H 9's notification section differs for the non-resident electoral
    /// college and when neither a representative nor a postal address is given.
    #[test]
    fn h9_notification_branches() {
        // Neither representative nor postal address: "niet van toepassing".
        let mut input = read_example("model-h9-example-1.json");
        input["detailed_candidate"]["postal_address"] = Value::Null;
        input["detailed_candidate"]["bsn"] = Value::Null;
        assert_pdf(&render("model-h9", input), "h9 without address");

        // Non-resident electoral college: digital-notification consent.
        let mut input = read_example("model-h9-example-1.json");
        input["election_type"] = json!("KCNI");
        assert_pdf(&render("model-h9", input), "h9 KCNI");
    }

    /// I 4 is only reachable through `render_example`; render it with every
    /// section empty so the "geen ..." fallbacks run, and with the objections
    /// still open so the write-in space is emitted.
    #[test]
    fn i4_renders_with_empty_sections() {
        let mut input = read_example("model-i4-example-1.json");
        for key in [
            "found_omissions",
            "recovered_omissions",
            "invalid_lists",
            "removed_candidates",
            "removed_designations",
            "corrected_designations",
        ] {
            input[key] = json!([]);
        }
        input["objections"] = json!([]);
        input["response_objections"] = Value::Null;
        assert_pdf(&render("model-i4", input), "i4 empty sections");

        let mut input = read_example("model-i4-example-1.json");
        input["objections"] = Value::Null;
        assert_pdf(&render("model-i4", input), "i4 open objections");
    }

    /// Models report a download file name; check the locale- and
    /// designation-dependent ones, including I 4 which the app never renders.
    #[test]
    fn filenames() {
        let combined: h3::H3 =
            serde_json::from_value(read_example("model-h3-2-example-1.json")).unwrap();
        assert_eq!(combined.filename(), "h3-2-samengevoegde-aanduiding.pdf");

        let mut frisian = read_example("model-h3-1-example-1.json");
        frisian["locale"] = json!("fry");
        let frisian: h3::H3 = serde_json::from_value(frisian).unwrap();
        assert_eq!(frisian.filename(), "h3-1-oantsjutting.pdf");

        let proces_verbaal: i4::I4 =
            serde_json::from_value(read_example("model-i4-example-1.json")).unwrap();
        assert_eq!(proces_verbaal.filename(), "i4-proces-verbaal.pdf");
    }
}
