use crate::port::assistant::{LetterDraft, LetterKind, LetterLanguage, LetterRequest};

/// The fixed words of a template letter in one language.
struct Wording {
    salutation: &'static str,
    closing: &'static str,
}

const GERMAN: Wording = Wording { salutation: "Sehr geehrte Damen und Herren,", closing: "Mit freundlichen Grüßen" };
const ENGLISH: Wording = Wording { salutation: "Dear Sir or Madam,", closing: "Kind regards" };

/// A letter without AI: a fixed text per kind and language, with the facts the feature knows listed in the body and
/// the first of them in the subject. The user edits it before saving.
pub fn template_draft(request: &LetterRequest) -> LetterDraft {
    let wording = match request.language {
        LetterLanguage::De => GERMAN,
        LetterLanguage::En => ENGLISH,
    };
    let (subject, request_text) = phrases(request.kind, request.language);
    let subject = match request.subject_facts.first() {
        Some(fact) => format!("{subject}: {fact}"),
        None => subject.to_owned(),
    };
    let mut parts = vec![wording.salutation.to_owned(), request_text.to_owned()];
    if !request.subject_facts.is_empty() {
        parts.push(request.subject_facts.join("\n"));
    }
    let signature = request.sender.as_ref().map(|sender| sender.name.as_str()).unwrap_or_default();
    parts.push(format!("{}\n\n{signature}", wording.closing).trim_end().to_owned());
    LetterDraft { subject, body: parts.join("\n\n") }
}

/// The subject and the main sentences of a letter of `kind`.
fn phrases(kind: LetterKind, language: LetterLanguage) -> (&'static str, &'static str) {
    match (kind, language) {
        (LetterKind::Cancellation, LetterLanguage::De) => (
            "Kündigung",
            "hiermit kündige ich den Vertrag fristgerecht zum nächstmöglichen Zeitpunkt. Bitte bestätigen Sie mir \
             die Kündigung und das Vertragsende schriftlich.",
        ),
        (LetterKind::Cancellation, LetterLanguage::En) => (
            "Cancellation",
            "I hereby cancel the contract at the next possible date. Please confirm the cancellation and the end \
             date in writing.",
        ),
        (LetterKind::Objection, LetterLanguage::De) => (
            "Widerspruch",
            "hiermit widerspreche ich Ihrem Schreiben. Bitte prüfen Sie den Vorgang und teilen Sie mir das \
             Ergebnis schriftlich mit.",
        ),
        (LetterKind::Objection, LetterLanguage::En) => (
            "Objection",
            "I hereby object to your letter. Please review the matter and let me know the result in writing.",
        ),
        (LetterKind::Reminder, LetterLanguage::De) => (
            "Erinnerung",
            "auf mein Schreiben in dieser Angelegenheit habe ich leider noch keine Antwort erhalten. Bitte melden \
             Sie sich innerhalb von 14 Tagen.",
        ),
        (LetterKind::Reminder, LetterLanguage::En) => {
            ("Reminder", "I have not yet received an answer to my letter in this matter. Please reply within 14 days.")
        }
        (LetterKind::Request, LetterLanguage::De) => {
            ("Anfrage", "in dieser Angelegenheit bitte ich Sie um die nötigen Unterlagen oder eine kurze Auskunft.")
        }
        (LetterKind::Request, LetterLanguage::En) => {
            ("Request", "In this matter, could you please send me the documents needed or a short answer?")
        }
        (LetterKind::Reply, LetterLanguage::De) => {
            ("Antwort", "vielen Dank für Ihr Schreiben. Dazu teile ich Ihnen Folgendes mit:")
        }
        (LetterKind::Reply, LetterLanguage::En) => ("Reply", "Thank you for your letter. My answer is as follows:"),
    }
}
