use crate::field::FieldKey;
use crate::restructure::PlaceholderName;

/// A piece of one folder name in a [`crate::restructure::PathTemplate`]: fixed text or a placeholder.
///
/// Only [`PlaceholderName::Field`] takes a `field_key`; [`crate::restructure::PathTemplate::parse`], the only
/// place that builds parts, keeps that rule.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TemplatePart {
    Literal(String),
    /// A value of the document; `fallback` is used when the document has none.
    Placeholder {
        name: PlaceholderName,
        field_key: Option<FieldKey>,
        fallback: Option<String>,
    },
}
