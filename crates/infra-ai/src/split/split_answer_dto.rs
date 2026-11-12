use serde::Deserialize;
use serde_json::{Value, json};

use crate::classify::suggestion_dto::{lenient_text, null_as_default};

/// The AI's parts, as [`split_schema`] asks for them; page numbers count from 1.
#[derive(Debug, Deserialize)]
pub(crate) struct SplitAnswerDto {
    #[serde(deserialize_with = "null_as_default")]
    pub parts: Vec<SplitPartDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SplitPartDto {
    pub first_page: usize,
    pub last_page: usize,
    #[serde(default, deserialize_with = "lenient_text")]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub category: Option<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub correspondent: Option<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub document_date: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
}

/// The strict JSON schema of a split: the parts in page order.
pub(crate) fn split_schema() -> Value {
    let text = json!({"type": ["string", "null"]});
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["parts"],
        "properties": {
            "parts": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["firstPage", "lastPage", "title", "category", "correspondent", "documentDate",
                        "confidence"],
                    "properties": {
                        "firstPage": {"type": "integer"},
                        "lastPage": {"type": "integer"},
                        "title": text,
                        "category": text,
                        "correspondent": text,
                        "documentDate": text,
                        "confidence": {"type": ["number", "null"]},
                    },
                },
            },
        },
    })
}
