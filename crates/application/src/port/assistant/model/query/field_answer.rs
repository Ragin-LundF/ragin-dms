use dms_domain::query::FieldComparison;

/// A condition on a custom field as the AI wrote it: the key, the comparison and the value as text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldAnswer {
    pub key: String,
    pub comparison: FieldComparison,
    pub value: Option<String>,
}
