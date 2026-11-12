/// One page of a stack as the AI gets it: its text, already cut short, and whether it looked blank (a separator
/// sheet or an empty back side).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitPage {
    pub text: String,
    pub blank: bool,
}
