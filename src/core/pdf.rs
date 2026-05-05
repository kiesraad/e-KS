use crate::{AppError, core::TypstRenderer};
use serde::Serialize;

const PDF_CONTENT_TYPE: &str = "application/pdf";
const ZIP_CONTENT_TYPE: &str = "application/zip";

pub trait Pdf: Sized + Serialize {
    fn typst_template_name(&self) -> &'static str;

    fn filename(&self) -> &str;

    async fn generate_bytes(&self, renderer: &TypstRenderer) -> Result<Vec<u8>, AppError> {
        renderer
            .render_pdf(self.typst_template_name(), self.filename(), &self)
            .await
    }
}
