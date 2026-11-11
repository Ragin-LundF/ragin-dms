use crate::library::FileNameViolation;
use crate::text;

/// The only validator for names of new files and folders, whether typed by the user or suggested by the AI.
///
/// A library is spread over macOS, Windows, Linux and network shares, so a name must be valid on all of them:
/// the strictest rules (Windows) apply everywhere. Names are also compared the way the case-insensitive file
/// systems compare them, so two siblings never differ only by case.
pub struct FileNamePolicy;

const FIRST_SUFFIX: u32 = 2;
const FORBIDDEN_CHARACTERS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
const RESERVED_NAMES: [&str; 4] = ["CON", "PRN", "AUX", "NUL"];

impl FileNamePolicy {
    /// Conservative limit (in UTF-16 units, as in Kotlin) that leaves room for the folder path within
    /// Windows' path length.
    pub const MAX_LENGTH: usize = 200;

    /// Returns the first rule `name` breaks, or `None` if it is a valid name.
    #[must_use]
    pub fn check(name: &str) -> Option<FileNameViolation> {
        let (Some(first), Some(last)) = (name.chars().next(), name.chars().last()) else {
            return Some(FileNameViolation::Empty);
        };
        if text::utf16_len(name) > Self::MAX_LENGTH {
            Some(FileNameViolation::TooLong)
        } else if name.chars().any(is_control_character) {
            Some(FileNameViolation::ControlCharacter)
        } else if name.chars().any(|character| FORBIDDEN_CHARACTERS.contains(&character)) {
            Some(FileNameViolation::ForbiddenCharacter)
        } else if first == '.' || text::is_whitespace(first) {
            Some(FileNameViolation::LeadingDotOrSpace)
        } else if last == '.' || text::is_whitespace(last) {
            Some(FileNameViolation::TrailingDotOrSpace)
        } else if is_reserved(name) {
            Some(FileNameViolation::ReservedName)
        } else {
            None
        }
    }

    /// `true` if a case-insensitive, Unicode-normalizing file system would treat both as one name.
    #[must_use]
    pub fn is_same_name(first: &str, second: &str) -> bool {
        Self::comparison_key(first) == Self::comparison_key(second)
    }

    /// `true` if `name` would clash with one of `siblings` in the same folder.
    pub fn collides_with_any<S: AsRef<str>>(name: &str, siblings: impl IntoIterator<Item = S>) -> bool {
        siblings.into_iter().any(|sibling| Self::is_same_name(sibling.as_ref(), name))
    }

    /// Returns `name` if it is free among `siblings`, otherwise the first free `name (n).ext` with n starting
    /// at 2.
    #[must_use]
    pub fn next_free_name<S: AsRef<str>>(name: &str, siblings: &[S]) -> String {
        if !Self::collides_with_any(name, siblings) {
            return name.to_owned();
        }
        let (base, extension) = match name.rfind('.') {
            Some(dot) if dot > 0 => name.split_at(dot),
            _ => (name, ""),
        };
        // Finite siblings leave a free counter, so the search ends.
        #[allow(clippy::maybe_infinite_iter)]
        (FIRST_SUFFIX..)
            .map(|counter| format!("{base} ({counter}){extension}"))
            .find(|candidate| !Self::collides_with_any(candidate, siblings))
            .unwrap_or_default()
    }

    /// `name` turned into a valid name: forbidden and control characters become `-`, leading dots and blanks
    /// and trailing dots and blanks are removed, reserved names get a trailing `_`, and the length is cut to
    /// [`Self::MAX_LENGTH`]. A name with nothing usable left becomes `_`.
    #[must_use]
    pub fn sanitize(name: &str) -> String {
        let replaced: String = text::nfc(name)
            .chars()
            .map(|character| {
                if FORBIDDEN_CHARACTERS.contains(&character) || is_control_character(character) {
                    '-'
                } else {
                    character
                }
            })
            .collect();
        let is_trimmed = |character: char| character == '.' || text::is_whitespace(character);
        let trimmed =
            text::take_utf16(replaced.trim_start_matches(is_trimmed), Self::MAX_LENGTH).trim_end_matches(is_trimmed);
        let usable = if trimmed.is_empty() { "_" } else { trimmed };
        if is_reserved(usable) { format!("{usable}_") } else { usable.to_owned() }
    }

    /// The domain's rule of "same name": two names, or two paths, are the same on a case-insensitive,
    /// Unicode-normalizing file system exactly when their keys are equal.
    #[must_use]
    pub fn comparison_key(name: &str) -> String {
        text::lowercase(&text::nfc(name))
    }
}

fn is_control_character(character: char) -> bool {
    (character as u32) < 32 || character as u32 == 127
}

fn is_reserved(name: &str) -> bool {
    let base_name = text::trim_end(name.split('.').next().unwrap_or_default()).to_uppercase();
    RESERVED_NAMES.contains(&base_name.as_str()) || is_numbered_device(&base_name)
}

/// `COM1`..`COM9` and `LPT1`..`LPT9`.
fn is_numbered_device(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() == 4 && (name.starts_with("COM") || name.starts_with("LPT")) && (b'1'..=b'9').contains(&bytes[3])
}
