use crate::DomainError;
use crate::error::require;
use crate::field::{FieldKey, FieldRole, FieldType};
use crate::text;

/// A custom field the library uses, such as "Amount" or "Cancel by". Only date fields can carry a deadline
/// [`Self::role`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldDefinition {
    pub key: FieldKey,
    pub label: String,
    pub field_type: FieldType,
    pub role: Option<FieldRole>,
}

impl FieldDefinition {
    /// # Errors
    /// Fails like [`Self::validate`].
    pub fn new(
        key: FieldKey,
        label: impl Into<String>,
        field_type: FieldType,
        role: Option<FieldRole>,
    ) -> Result<Self, DomainError> {
        let definition = Self { key, label: label.into(), field_type, role };
        definition.validate()?;
        Ok(definition)
    }

    /// # Errors
    /// Fails for a blank label or a deadline role on a field that is no date.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(!text::is_blank(&self.label), || "Field label must not be blank".to_owned())?;
        require(self.role.is_none() || self.field_type == FieldType::Date, || {
            "Only date fields can be deadlines".to_owned()
        })
    }
}
