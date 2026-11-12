use crate::DomainError;
use crate::contract::ContractId;
use crate::document::DocumentId;
use crate::error::{require_none_or_not_blank, require_not_blank};

/// What an item of the emergency folder holds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmergencyEntry {
    Document(DocumentId),
    Contract(ContractId),
    Contact { name: String, role: Option<String>, phone: Option<String>, email: Option<String>, note: Option<String> },
    Note(String),
}

impl EmergencyEntry {
    /// # Errors
    /// Fails for a contact without a name, blank contact details or a blank note.
    pub fn validate(&self) -> Result<(), DomainError> {
        match self {
            Self::Document(_) | Self::Contract(_) => Ok(()),
            Self::Contact { name, role, phone, email, note } => {
                require_not_blank(name, "Contact name")?;
                require_none_or_not_blank(role.as_deref(), "Role")?;
                require_none_or_not_blank(phone.as_deref(), "Phone")?;
                require_none_or_not_blank(email.as_deref(), "Email")?;
                require_none_or_not_blank(note.as_deref(), "Note")
            }
            Self::Note(text) => require_not_blank(text, "Note"),
        }
    }
}
