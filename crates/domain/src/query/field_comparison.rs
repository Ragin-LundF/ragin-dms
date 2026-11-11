/// How a [`crate::query::FieldCondition`] compares a document's field value. [`Self::WithinDays`] takes a number
/// of days and matches dates from today up to that many days ahead, so a saved search stays current.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldComparison {
    Present,
    Absent,
    Equals,
    Contains,
    LessThan,
    GreaterThan,
    WithinDays,
}
