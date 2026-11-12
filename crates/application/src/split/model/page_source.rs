/// Where the page texts of a stack came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageSource {
    TextLayer,
    /// The AI's stored transcription, which kept the page boundaries.
    Transcription,
    /// The vision model read every page that was not blank.
    Vision,
}
