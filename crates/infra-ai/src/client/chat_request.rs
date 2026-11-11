//! The body of `POST /chat/completions`: the messages, and for a structured answer the JSON schema.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use dms_application::port::ai::PageImage;
use serde_json::{Value, json};

/// One message of a chat request; `content` is a string or a list of text and image parts.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ChatMessage {
    pub role: &'static str,
    pub content: Value,
}

impl ChatMessage {
    pub(crate) fn system(text: &str) -> Self {
        Self { role: "system", content: Value::String(text.to_owned()) }
    }

    pub(crate) fn user(text: &str) -> Self {
        Self { role: "user", content: Value::String(text.to_owned()) }
    }

    /// A user message with the page images after the text.
    pub(crate) fn user_with_images(text: &str, images: &[PageImage]) -> Self {
        Self { role: "user", content: content(text, images) }
    }
}

/// A chat whose answer the server constrains to `schema` (strict).
pub(crate) fn structured(
    model: &str,
    temperature: f64,
    messages: &[ChatMessage],
    schema_name: &str,
    schema: &Value,
) -> Value {
    let mut body = chat(model, temperature, messages);
    body["response_format"] = json!({
        "type": "json_schema",
        "json_schema": {"name": schema_name, "strict": true, "schema": schema},
    });
    body
}

/// A plain chat, answered with free text.
pub(crate) fn chat(model: &str, temperature: f64, messages: &[ChatMessage]) -> Value {
    let messages: Vec<Value> =
        messages.iter().map(|message| json!({"role": message.role, "content": message.content})).collect();
    json!({"model": model, "temperature": temperature, "stream": false, "messages": messages})
}

/// Text alone as one string; with page images, a text part followed by one image part per page.
pub(crate) fn content(text: &str, images: &[PageImage]) -> Value {
    if images.is_empty() {
        return Value::String(text.to_owned());
    }
    let mut parts = vec![json!({"type": "text", "text": text})];
    parts.extend(images.iter().map(|page| {
        let url = format!("data:{};base64,{}", page.mime_type, STANDARD.encode(&page.bytes));
        json!({"type": "image_url", "image_url": {"url": url}})
    }));
    Value::Array(parts)
}
