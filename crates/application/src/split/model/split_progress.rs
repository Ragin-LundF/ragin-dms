/// The vision model read `done` of the `total` pages of a stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitProgress {
    pub done: usize,
    pub total: usize,
}
