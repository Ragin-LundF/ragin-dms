use serde::Deserialize;
use serde_json::{Value, json};

use crate::classify::suggestion_dto::{lenient_text, null_as_default};

/// The AI's filters, as [`query_schema`] asks for them.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QueryAnswerDto {
    #[serde(default, deserialize_with = "lenient_text")]
    pub text: Option<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub category: Option<String>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub tags: Vec<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub correspondent: Option<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub date_from: Option<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub date_to: Option<String>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub fields: Vec<FieldConditionDto>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub payment: Option<String>,
    #[serde(default)]
    pub tax_year: Option<i32>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub tax_area: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FieldConditionDto {
    pub key: String,
    pub comparison: String,
    #[serde(default, deserialize_with = "lenient_text")]
    pub value: Option<String>,
}

/// The strict JSON schema of the filters; every property is required, "not asked for" is `null` or `[]`.
pub(crate) fn query_schema() -> Value {
    let text = json!({"type": ["string", "null"]});
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["text", "category", "tags", "correspondent", "dateFrom", "dateTo", "fields", "payment",
            "taxYear", "taxArea"],
        "properties": {
            "text": text,
            "category": text,
            "tags": {"type": "array", "items": {"type": "string"}},
            "correspondent": text,
            "dateFrom": text,
            "dateTo": text,
            "fields": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["key", "comparison", "value"],
                    "properties": {
                        "key": {"type": "string"},
                        "comparison": {
                            "type": "string",
                            "enum": ["equals", "contains", "lessThan", "greaterThan", "present", "absent"],
                        },
                        "value": text,
                    },
                },
            },
            "payment": text,
            "taxYear": {"type": ["integer", "null"]},
            "taxArea": text,
        },
    })
}
