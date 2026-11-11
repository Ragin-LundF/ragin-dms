use serde_json::{Value, json};

const BOX_SIZE: usize = 4;

/// The strict JSON schema of a page layout: the text lines and the graphics, each with its box.
pub(crate) fn layout_schema() -> Value {
    let bounds = json!({"type": "array", "items": {"type": "integer"}, "minItems": BOX_SIZE, "maxItems": BOX_SIZE});
    let line = json!({
        "type": "object",
        "properties": {
            "bbox": bounds,
            "text": {"type": "string"},
            "bold": {"type": "boolean"},
            "font": {"type": "string", "enum": ["sans", "serif", "mono"]},
            "legible": {"type": "boolean"},
        },
        "required": ["bbox", "text", "bold", "font", "legible"],
        "additionalProperties": false,
    });
    let graphic = json!({
        "type": "object",
        "properties": {"kind": {"type": "string"}, "bbox": bounds},
        "required": ["kind", "bbox"],
        "additionalProperties": false,
    });
    json!({
        "type": "object",
        "properties": {
            "lines": {"type": "array", "items": line},
            "graphics": {"type": "array", "items": graphic},
        },
        "required": ["lines", "graphics"],
        "additionalProperties": false,
    })
}
