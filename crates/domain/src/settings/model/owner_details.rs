use crate::DomainError;
use crate::error::{require, require_none_or_not_blank, require_not_blank};
use crate::text;

/// Who the library belongs to, for the sender block of letters and the emergency overview; shared across devices.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwnerDetails {
    pub name: String,
    pub address_lines: Vec<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

impl OwnerDetails {
    /// # Errors
    /// Fails for a blank name, a blank address line, or a blank email or phone.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_not_blank(&self.name, "Your name")?;
        require(self.address_lines.iter().all(|line| !text::is_blank(line)), || {
            "An address line must not be blank".to_owned()
        })?;
        require_none_or_not_blank(self.email.as_deref(), "Email")?;
        require_none_or_not_blank(self.phone.as_deref(), "Phone")
    }
}
