use crate::library::RelativePath;
use crate::restructure::PlaceholderName;

/// The folder a [`crate::restructure::PathTemplate`] gives one document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateRendering {
    Rendered(RelativePath),
    /// The document has no value for the placeholder and the template gives no fallback.
    Missing(PlaceholderName),
}
