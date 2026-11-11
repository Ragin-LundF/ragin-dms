use crate::library::RelativePath;
use crate::text;

/// What kind of content a file holds, decided by its extension.
///
/// [`DocumentKind::Other`] files are catalogued and synchronized like every other file, but never sent to the AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentKind {
    Pdf,
    Image,
    Office,
    Other,
}

impl DocumentKind {
    #[must_use]
    pub fn readable_by_ai(self) -> bool {
        self != Self::Other
    }

    /// The Kotlin enum constant name, such as `PDF`; restructure templates print it.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Pdf => "PDF",
            Self::Image => "IMAGE",
            Self::Office => "OFFICE",
            Self::Other => "OTHER",
        }
    }

    #[must_use]
    pub fn of(path: &RelativePath) -> Self {
        match text::lowercase(text::after_last(path.file_name(), '.')).as_str() {
            "pdf" => Self::Pdf,
            "jpg" | "jpeg" | "png" => Self::Image,
            "docx" | "pptx" | "xlsx" => Self::Office,
            _ => Self::Other,
        }
    }
}
