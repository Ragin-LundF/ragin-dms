/// Where an item of a tax year's checklist stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaxItemState {
    /// A document of the year is assigned to it.
    Found,
    /// The usual date of the document is still ahead.
    Expected,
    Missing,
    /// The user marked it as not needed this year.
    NotNeeded,
}
