use crate::field::{FieldDefinition, FieldKey, FieldRole, FieldType};

/// The fields a library gets when it uses custom fields for the first time.
pub struct FieldPresets;

impl FieldPresets {
    #[must_use]
    pub fn amount() -> FieldDefinition {
        preset("amount", "Amount", FieldType::Money, None)
    }

    #[must_use]
    pub fn due_date() -> FieldDefinition {
        preset("due", "Due date", FieldType::Date, Some(FieldRole::DueDate))
    }

    #[must_use]
    pub fn cancel_by() -> FieldDefinition {
        preset("cancel-by", "Cancel by", FieldType::Date, Some(FieldRole::CancelBy))
    }

    #[must_use]
    pub fn contract_end() -> FieldDefinition {
        preset("contract-end", "Contract end", FieldType::Date, Some(FieldRole::ContractEnd))
    }

    #[must_use]
    pub fn warranty_end() -> FieldDefinition {
        preset("warranty-until", "Warranty until", FieldType::Date, Some(FieldRole::WarrantyEnd))
    }

    #[must_use]
    pub fn all() -> Vec<FieldDefinition> {
        vec![Self::amount(), Self::due_date(), Self::cancel_by(), Self::contract_end(), Self::warranty_end()]
    }
}

fn preset(key: &str, label: &str, field_type: FieldType, role: Option<FieldRole>) -> FieldDefinition {
    FieldDefinition {
        key: FieldKey::new(key).expect("invariant: preset keys are valid"),
        label: label.to_owned(),
        field_type,
        role,
    }
}
