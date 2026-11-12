/// What the user decided about a sender's rhythm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpectationMode {
    /// Never expect documents from it.
    Off,
    /// Keep expecting documents, also after two missed periods.
    Keep,
}
