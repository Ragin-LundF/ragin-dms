/// One passage the answer may draw on, numbered as it is cited: `[number]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceChunk {
    pub number: usize,
    pub title: String,
    pub text: String,
}
