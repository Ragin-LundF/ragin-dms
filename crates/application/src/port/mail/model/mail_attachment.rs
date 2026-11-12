/// A file attached to a mail, as its sender named it. Inline pictures of an HTML body are not attachments.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MailAttachment {
    pub name: String,
    pub size: u64,
}
