use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use pdfium_render::prelude::{PdfDocument, Pdfium};

use crate::error::PdfError;

/// PDFium can be bound only once per process, so the first successful [`PdfEngine::bind`] wins.
static PDFIUM: OnceLock<Pdfium> = OnceLock::new();
static BINDING: Mutex<()> = Mutex::new(());

/// The PDF engine on PDFium. Cheap to copy; every operation is synchronous, so callers run it on a
/// blocking thread. Sources are only read; every result is a new file.
#[derive(Clone, Copy)]
pub struct PdfEngine {
    pdfium: &'static Pdfium,
}

impl PdfEngine {
    /// Loads the PDFium library from [`library`]: the library file itself, or a folder that holds it
    /// directly or in `lib/` or `bin/` (the layout of the prebuilt binaries). The Tauri app passes its
    /// resource folder. Later calls return the engine already bound.
    ///
    /// # Errors
    /// [`PdfError::Library`] if the library cannot be found or loaded.
    pub fn bind(library: &Path) -> Result<Self, PdfError> {
        if let Some(pdfium) = PDFIUM.get() {
            return Ok(PdfEngine { pdfium });
        }
        let _guard = BINDING.lock().map_err(|_| PdfError::Library("binding panicked before".into()))?;
        if let Some(pdfium) = PDFIUM.get() {
            return Ok(PdfEngine { pdfium });
        }
        let file =
            library_file(library).ok_or_else(|| PdfError::Library(format!("no PDFium in {}", library.display())))?;
        let bindings = Pdfium::bind_to_library(&file).map_err(|error| PdfError::Library(format!("{error:?}")))?;
        Ok(PdfEngine { pdfium: PDFIUM.get_or_init(|| Pdfium::new(bindings)) })
    }

    /// The development copy fetched by `scripts/fetch-pdfium.sh` for this machine, for tests and
    /// `cargo run`; `None` if it was not fetched.
    #[must_use]
    pub fn dev_library() -> Option<PathBuf> {
        let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "aarch64") => "mac-arm64",
            ("macos", "x86_64") => "mac-x64",
            ("windows", "x86_64") => "win-x64",
            ("linux", "x86_64") => "linux-x64",
            _ => return None,
        };
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/pdfium").join(platform);
        library_file(&folder)
    }

    /// Opens an existing PDF for reading. A missing file is [`PdfError::NotFound`]; anything PDFium
    /// cannot open (broken, not a PDF, password protected) is [`PdfError::UnsupportedFormat`].
    pub(crate) fn open(self, path: &Path) -> Result<PdfDocument<'static>, PdfError> {
        if !path.is_file() {
            return Err(PdfError::NotFound);
        }
        self.pdfium.load_pdf_from_file(path, None).map_err(PdfError::from)
    }

    pub(crate) fn create(self) -> Result<PdfDocument<'static>, PdfError> {
        self.pdfium.create_new_pdf().map_err(PdfError::from)
    }
}

fn library_file(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    let name = Pdfium::pdfium_platform_library_name();
    [path.join(&name), path.join("lib").join(&name), path.join("bin").join(&name)]
        .into_iter()
        .find(|candidate| candidate.is_file())
}
