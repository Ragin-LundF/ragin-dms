//! The prompts of a letter: the system part names the language, the request carries the facts and the documents.

use dms_application::port::assistant::{LetterKind, LetterLanguage, LetterParty, LetterRequest};

use crate::prompt;

const NONE: &str = "none";

pub(crate) fn system(request: &LetterRequest) -> String {
    let language = match request.language {
        LetterLanguage::De => "German",
        LetterLanguage::En => "English",
    };
    prompt::LETTER.fill(&[("language", language)])
}

pub(crate) fn request(request: &LetterRequest) -> String {
    let kind = match request.kind {
        LetterKind::Cancellation => "cancellation of a contract",
        LetterKind::Objection => "objection to a decision, bill or statement",
        LetterKind::Reminder => "reminder about an earlier letter that got no answer",
        LetterKind::Request => "request for a document or information",
        LetterKind::Reply => "reply to a letter",
    };
    let sender = request.sender.as_ref().map_or_else(|| NONE.to_owned(), party);
    let facts = lines_or_none(&request.subject_facts, "\n");
    let documents = lines_or_none(&request.source_texts, "\n\n---\n\n");
    prompt::LETTER_REQUEST.fill(&[
        ("kind", kind),
        ("sender", &sender),
        ("recipient", &party(&request.recipient)),
        ("facts", &facts),
        ("documents", &documents),
    ])
}

fn party(party: &LetterParty) -> String {
    let lines: Vec<&str> = std::iter::once(&party.name)
        .chain(&party.address_lines)
        .map(String::as_str)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.is_empty() { "unknown".to_owned() } else { lines.join(", ") }
}

fn lines_or_none(lines: &[String], separator: &str) -> String {
    if lines.is_empty() { NONE.to_owned() } else { lines.join(separator) }
}
