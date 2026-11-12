use serde_json::{Map, Value, json};

const NULLABLE_TEXT: [&str; 12] = [
    "title",
    "category",
    "documentDate",
    "correspondent",
    "fileName",
    "transcription",
    "targetFolder",
    "targetReason",
    "taxArea",
    "deductible",
    "iban",
    "paymentReference",
];

/// The JSON schema of the answer, sent as `response_format`, so the server constrains the model to valid JSON.
/// Strict mode needs every property listed as required; "not known" is `null`.
pub(crate) fn suggestion_schema() -> Value {
    let mut properties = Map::new();
    for name in NULLABLE_TEXT {
        properties.insert(name.into(), json!({"type": ["string", "null"]}));
    }
    properties.insert("tags".into(), json!({"type": "array", "items": {"type": "string"}}));
    properties.insert("pages".into(), json!({"type": "array", "items": {"type": "string"}}));
    properties.insert("confidence".into(), json!({"type": ["number", "null"]}));
    properties.insert(
        "fields".into(),
        json!({
            "type": "array",
            "items": {
                "type": "object",
                "additionalProperties": false,
                "properties": {"key": {"type": "string"}, "value": {"type": ["string", "null"]}},
                "required": ["key", "value"],
            },
        }),
    );
    properties.insert("targetFolderIsNew".into(), json!({"type": ["boolean", "null"]}));
    properties.insert("taxYear".into(), json!({"type": ["integer", "null"]}));
    properties.insert(
        "requestedActions".into(),
        json!({
            "type": "array",
            "items": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "text": {"type": "string"},
                    "dueDate": {"type": ["string", "null"]},
                    "references": {"type": "array", "items": {"type": "string"}},
                },
                "required": ["text", "dueDate", "references"],
            },
        }),
    );
    let required: Vec<&str> = NULLABLE_TEXT
        .into_iter()
        .chain(["tags", "confidence", "targetFolderIsNew", "fields", "pages", "taxYear", "requestedActions"])
        .collect();
    json!({"type": "object", "additionalProperties": false, "properties": properties, "required": required})
}
