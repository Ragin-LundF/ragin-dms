//! Reading email files (RFC 5322, `.eml`) with `mail-parser`: header, body as plain text, attachments.

use std::fs;
use std::path::Path;

use chrono::{DateTime, FixedOffset, NaiveDate};
use mail_parser::{Addr, MessageParser, MessagePart, MimeHeaders};

use crate::error::PdfError;
use crate::files::{self, FileFacts};

/// A mail as the import needs it. Texts are decoded (RFC 2047 words, character sets); `body` is the plain-text
/// body, or the HTML body turned into text when there is none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailFile {
    pub from_name: Option<String>,
    pub from_address: Option<String>,
    pub to: Vec<String>,
    pub subject: Option<String>,
    pub sent_at: Option<DateTime<FixedOffset>>,
    pub body: String,
    pub attachments: Vec<MailPart>,
}

/// One attachment with its name and decoded content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailPart {
    pub name: String,
    pub content: Vec<u8>,
}

/// Reads the mail in `file`; the file is never changed.
///
/// # Errors
/// [`PdfError::NotFound`] or [`PdfError::Io`] when the file cannot be read, [`PdfError::UnsupportedFormat`] when
/// it holds no mail.
pub fn read_mail(file: &Path) -> Result<MailFile, PdfError> {
    let bytes = fs::read(file).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => PdfError::NotFound,
        _ => PdfError::Io(error),
    })?;
    parse_mail(&bytes)
}

/// Writes an attachment's `content` as a new file at `target`, which must not exist yet.
///
/// # Errors
/// [`PdfError::AlreadyExists`] if `target` exists; [`PdfError::Io`] if it cannot be written.
pub fn write_attachment(content: &[u8], target: &Path) -> Result<FileFacts, PdfError> {
    files::write_new(target, |temp| fs::write(temp, content).map_err(PdfError::Io))
}

pub(crate) fn parse_mail(bytes: &[u8]) -> Result<MailFile, PdfError> {
    // mail-parser accepts any text; without a sender, subject or date it is no mail.
    let message = MessageParser::default()
        .parse(bytes)
        .filter(|message| message.from().is_some() || message.subject().is_some() || message.date().is_some());
    let message = message.ok_or_else(|| PdfError::unsupported("no email message"))?;
    let from = message.from().and_then(|from| from.first());
    let attachments = message
        .attachments()
        .filter(|part| !is_inline_picture(part))
        .enumerate()
        .map(|(index, part)| MailPart {
            name: part.attachment_name().map_or_else(|| format!("Attachment {}", index + 1), str::to_owned),
            content: part.contents().to_vec(),
        })
        .collect();
    Ok(MailFile {
        from_name: from.and_then(Addr::name).map(str::to_owned),
        from_address: from.and_then(Addr::address).map(str::to_owned),
        to: message.to().map(|to| to.iter().map(address_text).collect()).unwrap_or_default(),
        subject: message.subject().map(str::to_owned),
        sent_at: message.date().and_then(|date| {
            let seconds = (i32::from(date.tz_hour) * 3600 + i32::from(date.tz_minute) * 60)
                * if date.tz_before_gmt { -1 } else { 1 };
            NaiveDate::from_ymd_opt(i32::from(date.year), u32::from(date.month), u32::from(date.day))?
                .and_hms_opt(u32::from(date.hour), u32::from(date.minute), u32::from(date.second))?
                .and_local_timezone(FixedOffset::east_opt(seconds)?)
                .single()
        }),
        body: message.body_text(0).map(|body| body.trim().to_owned()).unwrap_or_default(),
        attachments,
    })
}

/// A picture the HTML body shows inline (`cid:` reference) is part of the body, not an attachment of its own.
fn is_inline_picture(part: &MessagePart<'_>) -> bool {
    let attached = part.content_disposition().is_some_and(mail_parser::ContentType::is_attachment);
    let picture = part.content_type().is_some_and(|content_type| content_type.ctype().eq_ignore_ascii_case("image"));
    picture && part.content_id().is_some() && !attached
}

fn address_text(address: &Addr<'_>) -> String {
    match (address.name(), address.address()) {
        (Some(name), Some(address)) => format!("{name} <{address}>"),
        (Some(single), None) | (None, Some(single)) => single.to_owned(),
        (None, None) => String::new(),
    }
}

#[cfg(test)]
pub(crate) mod tests;
