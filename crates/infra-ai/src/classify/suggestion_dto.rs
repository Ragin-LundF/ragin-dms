use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// The JSON the model answers with, as the suggestion schema describes it. Everything is optional: what the model
/// leaves out is simply not suggested. Like Kotlin's lenient reader, `null` stands for an empty list and a number or
/// boolean where text belongs is read as text; any other wrong type makes the whole answer invalid.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SuggestionDto {
    #[serde(default, deserialize_with = "lenient_text")]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub category: Option<String>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub tags: Vec<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub document_date: Option<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub correspondent: Option<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub file_name: Option<String>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub transcription: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub target_folder: Option<String>,
    #[serde(default)]
    pub target_folder_is_new: Option<bool>,
    #[serde(default, deserialize_with = "lenient_text")]
    pub target_reason: Option<String>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub fields: Vec<FieldAnswerDto>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub pages: Vec<String>,
}

/// One custom field value the model read from the document, as plain text.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct FieldAnswerDto {
    #[serde(default, deserialize_with = "null_as_default")]
    pub key: String,
    #[serde(default, deserialize_with = "lenient_text")]
    pub value: Option<String>,
}

impl SuggestionDto {
    /// The outermost JSON object of `content`: servers that ignore the schema sometimes wrap it in a Markdown code
    /// fence or add a sentence around it. `None` if there is none or it is no suggestion.
    pub(crate) fn parse(content: &str) -> Option<Self> {
        serde_json::from_str(outermost_object(content)?).ok()
    }
}

/// From the first `{` to the last `}`.
pub(crate) fn outermost_object(content: &str) -> Option<&str> {
    let start = content.find('{')?;
    let end = content.rfind('}')?;
    (end > start).then(|| &content[start..=end])
}

pub(crate) fn null_as_default<'de, D: Deserializer<'de>, T: Default + Deserialize<'de>>(
    deserializer: D,
) -> Result<T, D::Error> {
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

fn lenient_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(None),
        Value::String(text) => Ok(Some(text)),
        Value::Number(number) => Ok(Some(number.to_string())),
        Value::Bool(flag) => Ok(Some(flag.to_string())),
        other => Err(serde::de::Error::custom(format!("expected text, found {other}"))),
    }
}
