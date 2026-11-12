use chrono::NaiveDate;
use dms_domain::text;

use crate::port::assistant::{LetterLanguage, LetterParty};
use crate::port::pdf::{TextBlock, TextDocument};

/// A letter as it is printed: the sender in one line, the recipient's address, the date, the subject in bold and the
/// body, one paragraph per blank-line separated part.
pub fn letter_document(
    sender: Option<&LetterParty>,
    recipient: &LetterParty,
    date: NaiveDate,
    subject: &str,
    body: &str,
    language: LetterLanguage,
) -> TextDocument {
    let mut blocks = Vec::new();
    if let Some(sender) = sender.filter(|sender| !text::is_blank(&sender.name)) {
        let line = std::iter::once(&sender.name).chain(&sender.address_lines).map(String::as_str).collect::<Vec<_>>();
        blocks.extend([TextBlock::Paragraph(line.join(" · ")), TextBlock::Spacer]);
    }
    let address = std::iter::once(&recipient.name).chain(&recipient.address_lines).map(|line| text::trim(line));
    let address: Vec<&str> = address.filter(|line| !line.is_empty()).collect();
    if !address.is_empty() {
        blocks.extend([TextBlock::Paragraph(address.join("\n")), TextBlock::Spacer]);
    }
    let date = match language {
        LetterLanguage::De => date.format("%d.%m.%Y").to_string(),
        LetterLanguage::En => date.format("%-d %B %Y").to_string(),
    };
    blocks.extend([TextBlock::Paragraph(date), TextBlock::Spacer, TextBlock::Heading(text::trim(subject).to_owned())]);
    let normalized = body.replace("\r\n", "\n");
    blocks.extend(
        normalized
            .split("\n\n")
            .map(|part| part.trim_matches('\n'))
            .filter(|part| !text::is_blank(part))
            .map(|part| TextBlock::Paragraph(part.to_owned())),
    );
    TextDocument { blocks }
}
