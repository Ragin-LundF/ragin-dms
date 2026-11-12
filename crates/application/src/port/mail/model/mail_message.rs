use chrono::{DateTime, FixedOffset};

use crate::port::mail::MailAttachment;

/// What a mail says: its header, its body as plain text (an HTML body is turned into text, inline pictures are
/// dropped) and its attachments in their order. Texts are decoded (RFC 2047, character sets).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailMessage {
    pub from_name: Option<String>,
    pub from_address: Option<String>,
    /// The recipients as "Name <address>" or the address alone.
    pub to: Vec<String>,
    pub subject: Option<String>,
    /// When it was sent, in the sender's time zone.
    pub sent_at: Option<DateTime<FixedOffset>>,
    pub body: String,
    pub attachments: Vec<MailAttachment>,
}
