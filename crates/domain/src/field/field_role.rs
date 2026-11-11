/// Marks a date field as a deadline, which the dashboard announces [`Self::default_lead_days`] days ahead.
/// Retention is no role: it is strictly opt-in and kept apart from the fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldRole {
    DueDate,
    CancelBy,
    ContractEnd,
    WarrantyEnd,
}

impl FieldRole {
    #[must_use]
    pub fn default_lead_days(self) -> i32 {
        match self {
            Self::DueDate => 7,
            Self::CancelBy | Self::ContractEnd | Self::WarrantyEnd => 30,
        }
    }
}
