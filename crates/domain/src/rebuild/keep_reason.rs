/// Why a text line of a rebuilt page stayed pixels of the scan instead of becoming real text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeepReason {
    /// The AI was not sure about every character.
    NotSure,
    /// No ink was found where the AI placed the line.
    NoInk,
    /// A stamp, signature or handwriting covers the line.
    Covered,
    /// The font has no glyph for a character of the line.
    NotEncodable,
    /// The text is wider or narrower than the ink: words are missing or invented.
    TextMismatch,
    /// The AI lost characters, and reading the line again did not recover them.
    LostCharacters,
    /// The AI could not read the page, so the whole page stayed as it was.
    PageFailed,
}
