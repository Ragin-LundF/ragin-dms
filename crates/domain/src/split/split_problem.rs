/// Why proposed parts cannot split a stack as they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplitProblem {
    NoParts,
    /// A part starts after the page that follows the previous part, or the last part ends before the last page.
    Gap,
    /// A part starts before the previous one ended, or ends before it starts.
    Overlap,
    BeyondLastPage,
}
