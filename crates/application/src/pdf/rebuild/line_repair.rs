use dms_domain::text;

/// Some models write NUL or U+FFFD instead of a character (seen with umlauts under a JSON schema). A line read again
/// is taken only if it agrees with every character the first reading did get, so the second reading can fill the
/// gaps but never change the line.
pub struct LineRepair;

/// One piece of the expected line: a known character, or a run of n lost ones that may be 1..=n others.
enum Piece {
    Known(char),
    Lost(usize),
}

impl LineRepair {
    #[must_use]
    pub fn has_lost_characters(text: &str) -> bool {
        text.chars().any(is_lost)
    }

    /// The repaired line, or `None` if `reread` does not fit `text`.
    #[must_use]
    pub fn repaired(text: &str, reread: &str) -> Option<String> {
        let candidate = normalized(reread);
        let pieces = pieces(&normalized(text));
        let characters: Vec<char> = candidate.chars().collect();
        fits(&pieces, &characters).then_some(candidate)
    }
}

fn is_lost(character: char) -> bool {
    character == '\0' || character == '\u{FFFD}'
}

/// Kotlin `trim()`, then runs of regex `\s` (ASCII whitespace) collapsed to one space.
fn normalized(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut in_space = false;
    for character in text::trim(line).chars() {
        if text::is_regex_space(character) {
            if !in_space {
                result.push(' ');
            }
            in_space = true;
        } else {
            result.push(character);
            in_space = false;
        }
    }
    result
}

fn pieces(line: &str) -> Vec<Piece> {
    let mut pieces = Vec::new();
    for character in line.chars() {
        match (is_lost(character), pieces.last_mut()) {
            (true, Some(Piece::Lost(run))) => *run += 1,
            (true, _) => pieces.push(Piece::Lost(1)),
            (false, _) => pieces.push(Piece::Known(character)),
        }
    }
    pieces
}

/// Whole-string match like Kotlin's `Regex.matches`: a lost run takes 1..=n characters that are neither regex
/// whitespace nor lost; known characters must be equal.
fn fits(pieces: &[Piece], characters: &[char]) -> bool {
    match pieces.split_first() {
        None => characters.is_empty(),
        Some((Piece::Known(expected), rest)) => characters.first() == Some(expected) && fits(rest, &characters[1..]),
        Some((Piece::Lost(run), rest)) => {
            let fillable =
                characters.iter().take(*run).take_while(|c| !text::is_regex_space(**c) && !is_lost(**c)).count();
            (1..=fillable).any(|taken| fits(rest, &characters[taken..]))
        }
    }
}
