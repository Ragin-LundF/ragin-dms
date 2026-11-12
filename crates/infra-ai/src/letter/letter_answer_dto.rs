use serde::Deserialize;
use serde_json::{Value, json};

/// The AI's letter, as [`letter_schema`] asks for it.
#[derive(Debug, Deserialize)]
pub(crate) struct LetterAnswerDto {
    pub subject: String,
    pub body: String,
}

/// The strict JSON schema of a letter: a subject line and the body.
pub(crate) fn letter_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"subject": {"type": "string"}, "body": {"type": "string"}},
        "required": ["subject", "body"],
        "additionalProperties": false,
    })
}
