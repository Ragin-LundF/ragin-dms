//! String helpers that behave exactly like the Kotlin/JVM functions the domain rules were written with.
//! Kotlin strings are UTF-16: lengths, `take` and ordering count UTF-16 code units, and `isWhitespace`
//! differs slightly from Rust's [`char::is_whitespace`].

use std::cmp::Ordering;

use unicode_normalization::UnicodeNormalization;

/// Kotlin's `Char.isWhitespace()`: Java's `isWhitespace || isSpaceChar`.
#[must_use]
pub fn is_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{9}'..='\u{D}'
            | '\u{1C}'..='\u{20}'
            | '\u{A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

/// The whitespace of a Java regex `\s` without the Unicode flag.
#[must_use]
pub fn is_regex_space(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\u{B}' | '\u{C}' | '\r')
}

/// Kotlin's `String.trim()`.
#[must_use]
pub fn trim(text: &str) -> &str {
    text.trim_matches(is_whitespace)
}

/// Kotlin's `String.trimEnd()`.
#[must_use]
pub fn trim_end(text: &str) -> &str {
    text.trim_end_matches(is_whitespace)
}

/// Kotlin's `String.isBlank()`.
#[must_use]
pub fn is_blank(text: &str) -> bool {
    text.chars().all(is_whitespace)
}

/// Kotlin's `String.length`: UTF-16 code units.
#[must_use]
pub fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// Kotlin's `String.take(n)`, except that a surrogate pair is never split: a character that would only
/// half fit is left out.
#[must_use]
pub fn take_utf16(text: &str, units: usize) -> &str {
    let mut used = 0;
    for (index, character) in text.char_indices() {
        used += character.len_utf16();
        if used > units {
            return &text[..index];
        }
    }
    text
}

/// Kotlin's `String.compareTo`: UTF-16 code unit order.
#[must_use]
pub fn compare(first: &str, second: &str) -> Ordering {
    first.encode_utf16().cmp(second.encode_utf16())
}

/// Kotlin's `lowercase(Locale.ROOT)`.
#[must_use]
pub fn lowercase(text: &str) -> String {
    text.to_lowercase()
}

/// `Normalizer.normalize(text, NFC)`.
#[must_use]
pub fn nfc(text: &str) -> String {
    text.nfc().collect()
}

/// Kotlin's `substringAfterLast(delimiter, missingDelimiterValue = "")` for a character delimiter.
#[must_use]
pub fn after_last(text: &str, delimiter: char) -> &str {
    text.rfind(delimiter).map_or("", |index| &text[index + delimiter.len_utf8()..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kotlin_whitespace_includes_separators_but_not_next_line() {
        assert!(is_whitespace('\u{1F}'));
        assert!(is_whitespace('\u{A0}'));
        assert!(!is_whitespace('\u{85}'));
        assert!(!is_whitespace('x'));
    }

    #[test]
    fn take_counts_utf16_units_and_keeps_pairs_whole() {
        assert_eq!(take_utf16("abc", 2), "ab");
        assert_eq!(take_utf16("a\u{1F600}b", 2), "a");
        assert_eq!(take_utf16("a\u{1F600}b", 3), "a\u{1F600}");
        assert_eq!(utf16_len("a\u{1F600}"), 3);
    }

    #[test]
    fn ordering_follows_utf16_like_the_jvm() {
        assert_eq!(compare("\u{FF5E}", "\u{1F600}"), Ordering::Greater);
        assert_eq!(compare("a", "b"), Ordering::Less);
    }
}
