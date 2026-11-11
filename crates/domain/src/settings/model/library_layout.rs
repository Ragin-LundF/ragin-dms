use crate::DomainError;
use crate::error::require;
use crate::field::CurrencyCode;
use crate::settings::InboxSettings;
use crate::text;

/// How the library is organised: the [`Self::inbox`], the path template the restructure assistant saved last
/// (inbox filing uses it as a pre-fill), the currency money fields use when none is given, and whether approved
/// metadata is also written into PDF files ([`Self::embed_metadata`], off by default).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct LibraryLayout {
    pub inbox: InboxSettings,
    pub path_template: Option<String>,
    pub default_currency: Option<CurrencyCode>,
    pub embed_metadata: bool,
}

impl LibraryLayout {
    /// # Errors
    /// Fails for a blank path template or an invalid inbox.
    pub fn validate(&self) -> Result<(), DomainError> {
        self.inbox.validate()?;
        require(self.path_template.as_deref().is_none_or(|template| !text::is_blank(template)), || {
            "Path template must be null or not blank".to_owned()
        })
    }
}
