use chrono::NaiveDate;

/// Something a letter asks the person to do (send documents, pay, reply, appeal), by `due_date` if it names one.
/// `references` are the numbers or names of documents the request mentions, such as invoice numbers, used to find
/// matching documents in the library.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestedAction {
    pub text: String,
    pub due_date: Option<NaiveDate>,
    pub references: Vec<String>,
}
