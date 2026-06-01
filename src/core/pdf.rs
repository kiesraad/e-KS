use crate::{AppError, core::TypstRenderer};
use serde::Serialize;

pub trait Pdf: Sized + Serialize {
    fn typst_template_name(&self) -> &'static str;

    fn filename(&self) -> String;

    async fn generate_bytes(&self, renderer: &TypstRenderer) -> Result<Vec<u8>, AppError> {
        renderer
            .render_pdf(self.typst_template_name(), &self.filename(), &self)
            .await
    }
}
