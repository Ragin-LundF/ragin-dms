use serde::Deserialize;
use serde_json::{Value, json};

/// The AI's reading of a contract, as [`contract_terms_schema`] asks for it.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContractTermsDto {
    pub prices: Vec<PriceDto>,
    pub notice_count: Option<u16>,
    pub notice_unit: Option<String>,
    pub term_end: Option<String>,
    pub renews_by_months: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PriceDto {
    pub valid_from: String,
    pub amount: String,
    pub currency: String,
    pub period: String,
    pub source: Option<usize>,
}

/// The strict JSON schema of a contract's prices and terms.
pub(crate) fn contract_terms_schema() -> Value {
    let nullable = |kind: &str| json!({"type": [kind, "null"]});
    json!({
        "type": "object",
        "properties": {
            "prices": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "validFrom": {"type": "string"},
                        "amount": {"type": "string"},
                        "currency": {"type": "string"},
                        "period": {"type": "string", "enum": ["monthly", "quarterly", "halfYearly", "yearly"]},
                        "source": nullable("integer"),
                    },
                    "required": ["validFrom", "amount", "currency", "period", "source"],
                    "additionalProperties": false,
                },
            },
            "noticeCount": nullable("integer"),
            "noticeUnit": {"type": ["string", "null"], "enum": ["days", "weeks", "months", null]},
            "termEnd": nullable("string"),
            "renewsByMonths": nullable("integer"),
        },
        "required": ["prices", "noticeCount", "noticeUnit", "termEnd", "renewsByMonths"],
        "additionalProperties": false,
    })
}
