use async_zip::{Compression, ZipEntryBuilder, tokio::write::ZipFileWriter};
use tokio::io::AsyncWrite;

use crate::AppError;

/// Writer used to add files into a streaming ZIP archive.
pub struct ZipResponseWriter<W: AsyncWrite + Unpin> {
    inner: ZipFileWriter<W>,
}

impl<W: AsyncWrite + Unpin> ZipResponseWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            inner: ZipFileWriter::with_tokio(writer),
        }
    }

    pub async fn add_file(&mut self, name: &str, data: &[u8]) -> Result<(), AppError> {
        let builder = ZipEntryBuilder::new(name.into(), Compression::Deflate);

        self.inner
            .write_entry_whole(builder, data)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn finish(self) -> Result<(), AppError> {
        self.inner
            .close()
            .await
            .map(|_| ())
            .map_err(|_| AppError::InternalServerError)
    }
}
