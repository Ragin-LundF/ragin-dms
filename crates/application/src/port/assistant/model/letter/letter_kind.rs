/// What a letter is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LetterKind {
    Cancellation,
    Objection,
    Reminder,
    Request,
    Reply,
}
