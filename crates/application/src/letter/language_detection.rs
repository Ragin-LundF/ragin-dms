use dms_domain::text;

use crate::port::assistant::LetterLanguage;

/// Frequent short words that tell German from English text.
const GERMAN: &[&str] = &[
    "der", "die", "das", "und", "ist", "nicht", "mit", "für", "von", "den", "dem", "sie", "ihr", "ihre", "wir",
    "bitte", "sehr", "geehrte", "zum", "zur", "auf", "eine", "ein", "bei",
];
const ENGLISH: &[&str] = &[
    "the", "and", "is", "not", "with", "for", "from", "you", "your", "we", "please", "dear", "of", "to", "our", "are",
    "this", "that", "by", "on",
];

/// The language of a letter answering `texts`: German when they hold more German than English words, otherwise
/// English, also when there is no text at all.
pub fn detect_language(texts: &[String]) -> LetterLanguage {
    let (mut german, mut english) = (0_usize, 0_usize);
    for text in texts {
        let lower = text::lowercase(text);
        for word in lower.split(|character: char| !character.is_alphabetic()).filter(|word| !word.is_empty()) {
            german += usize::from(GERMAN.contains(&word) || word.contains(['ä', 'ö', 'ü', 'ß']));
            english += usize::from(ENGLISH.contains(&word));
        }
    }
    if german > english { LetterLanguage::De } else { LetterLanguage::En }
}
