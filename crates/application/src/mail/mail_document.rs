use chrono::NaiveDate;
use dms_domain::library::FileNamePolicy;
use dms_domain::text;

use crate::port::mail::MailMessage;
use crate::port::pdf::{TextBlock, TextDocument};

const NO_SUBJECT: &str = "(no subject)";
/// Room for the date in front and ".pdf" behind the subject in a file name.
const SUBJECT_IN_NAME: usize = FileNamePolicy::MAX_LENGTH - 20;

/// Whether `name` is an email file the import understands (`.eml`, any case).
pub fn is_mail_file(name: &str) -> bool {
    std::path::Path::new(name).extension().is_some_and(|extension| extension.eq_ignore_ascii_case("eml"))
}

/// `<YYYY-MM-DD> <subject>.pdf`, made a valid file name.
pub fn mail_file_name(date: NaiveDate, mail: &MailMessage) -> String {
    let name = format!("{date} {}", subject_of(mail).unwrap_or("Email"));
    format!("{}.pdf", FileNamePolicy::sanitize(text::take_utf16(&name, SUBJECT_IN_NAME)))
}

/// The mail as it is printed: the subject as heading, a header block (from, to, date, subject, attachments) and the
/// body, one paragraph per blank-line separated part.
pub fn mail_document(mail: &MailMessage) -> TextDocument {
    let subject = subject_of(mail).unwrap_or(NO_SUBJECT).to_owned();
    let from = match (&mail.from_name, &mail.from_address) {
        (Some(name), Some(address)) => format!("{name} <{address}>"),
        (Some(single), None) | (None, Some(single)) => single.clone(),
        (None, None) => String::new(),
    };
    let mut header = vec![("From".to_owned(), from), ("To".to_owned(), mail.to.join(", "))];
    if let Some(sent_at) = mail.sent_at {
        header.push(("Date".to_owned(), sent_at.format("%Y-%m-%d %H:%M %:z").to_string()));
    }
    header.push(("Subject".to_owned(), subject.clone()));
    if !mail.attachments.is_empty() {
        let names: Vec<&str> = mail.attachments.iter().map(|attachment| attachment.name.as_str()).collect();
        header.push(("Attachments".to_owned(), names.join(", ")));
    }
    let mut blocks = vec![TextBlock::Heading(subject), TextBlock::KeyValueTable(header), TextBlock::Spacer];
    let body = mail.body.replace("\r\n", "\n");
    blocks.extend(
        body.split("\n\n")
            .map(|part| part.trim_matches('\n'))
            .filter(|part| !text::is_blank(part))
            .map(|part| TextBlock::Paragraph(part.to_owned())),
    );
    TextDocument { blocks }
}

fn subject_of(mail: &MailMessage) -> Option<&str> {
    mail.subject.as_deref().map(text::trim).filter(|subject| !subject.is_empty())
}
