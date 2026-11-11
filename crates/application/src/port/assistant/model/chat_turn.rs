use crate::port::assistant::ChatRole;

/// One message of a conversation about a document.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChatTurn {
    pub role: ChatRole,
    pub text: String,
}
