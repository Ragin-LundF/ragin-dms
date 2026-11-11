use std::io;

use pdfium_render::prelude::PdfiumError;

/// Why a PDF operation failed. The adapters map these onto the port's storage failures:
/// [`PdfError::NotFound`] → `NOT_FOUND`, [`PdfError::AlreadyExists`] → `ALREADY_EXISTS`,
/// [`PdfError::UnsupportedFormat`] → `UNSUPPORTED_FORMAT`, [`PdfError::Io`] and [`PdfError::ProgramFailed`]
/// → `IO_ERROR`. [`PdfError::InvalidRequest`] is a caller bug (the Kotlin `require` failures).
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("the file or page does not exist")]
    NotFound,
    #[error("the target already exists")]
    AlreadyExists,
    /// Not a PDF, broken, encrypted with a password, or not readable for another reason.
    #[error("not a readable PDF: {0}")]
    UnsupportedFormat(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    /// An external program such as `ocrmypdf` did not produce a result.
    #[error("{0}")]
    ProgramFailed(String),
    /// The PDFium library could not be loaded.
    #[error("PDFium is not available: {0}")]
    Library(String),
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
}

impl PdfError {
    #[allow(clippy::needless_pass_by_value)] // used as `map_err(PdfError::unsupported)`
    pub(crate) fn unsupported(error: impl ToString) -> Self {
        PdfError::UnsupportedFormat(error.to_string())
    }
}

impl From<PdfiumError> for PdfError {
    fn from(error: PdfiumError) -> Self {
        match error {
            PdfiumError::IoError(io) if io.kind() == io::ErrorKind::NotFound => PdfError::NotFound,
            PdfiumError::IoError(io) => PdfError::Io(io),
            other => PdfError::unsupported(format!("{other:?}")),
        }
    }
}
