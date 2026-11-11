use serde_json::{Map, Value, json};

const MAX_TOKENS: u32 = 16_000;
const DATA_URL_PREFIX: &str = "data:";
const BASE64_MARKER: &str = ";base64,";

/// Turns a chat completions body into a body for Anthropic's `POST /v1/messages`: system messages move to `system`,
/// image data URLs become base64 image blocks, and the answer schema becomes a structured output. `temperature` is
/// dropped, because current Claude models reject it.
pub(crate) fn anthropic_body(open_ai: &Value) -> Value {
    let messages = open_ai["messages"].as_array().map(Vec::as_slice).unwrap_or_default();
    let (system, chat): (Vec<&Value>, Vec<&Value>) =
        messages.iter().partition(|message| message["role"].as_str() == Some("system"));
    let mut body = Map::new();
    body.insert("model".into(), open_ai["model"].clone());
    body.insert("max_tokens".into(), json!(MAX_TOKENS));
    if !system.is_empty() {
        let text: Vec<&str> = system.iter().map(|message| message["content"].as_str().unwrap_or_default()).collect();
        body.insert("system".into(), Value::String(text.join("\n")));
    }
    let chat: Vec<Value> = chat
        .iter()
        .map(|message| json!({"role": message["role"], "content": content_of(&message["content"])}))
        .collect();
    body.insert("messages".into(), Value::Array(chat));
    if let Some(schema) = open_ai.pointer("/response_format/json_schema/schema") {
        body.insert("output_config".into(), json!({"format": {"type": "json_schema", "schema": schema}}));
    }
    Value::Object(body)
}

/// A string stays a string; text parts pass through and `image_url` parts become image blocks.
fn content_of(content: &Value) -> Value {
    let Some(parts) = content.as_array() else {
        return content.clone();
    };
    Value::Array(
        parts
            .iter()
            .map(|part| {
                let data = part.pointer("/image_url/url").and_then(Value::as_str).and_then(data_url_parts);
                match data {
                    Some((media_type, data)) => json!({
                        "type": "image",
                        "source": {"type": "base64", "media_type": media_type, "data": data},
                    }),
                    None => part.clone(),
                }
            })
            .collect(),
    )
}

/// The media type and the base64 data of a `data:<type>;base64,<data>` URL.
fn data_url_parts(url: &str) -> Option<(&str, &str)> {
    let (media_type, data) = url.strip_prefix(DATA_URL_PREFIX)?.split_once(BASE64_MARKER)?;
    (!media_type.is_empty() && !media_type.contains(';')).then_some((media_type, data))
}
