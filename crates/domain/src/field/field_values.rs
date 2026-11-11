use crate::dates;
use crate::field::{CurrencyCode, Decimal, FieldType, FieldValue};
use crate::text;

/// Reads and writes field values as plain text, the way the user types them and the AI answers. A value that
/// does not fit the field's type is refused with `None`, never guessed.
pub struct FieldValues;

impl FieldValues {
    /// `raw` as a value of `field_type`; a money value without a currency takes `default_currency`.
    #[must_use]
    pub fn parse(field_type: FieldType, raw: &str, default_currency: Option<&CurrencyCode>) -> Option<FieldValue> {
        let text = text::trim(raw);
        if text.is_empty() {
            return None;
        }
        match field_type {
            FieldType::Text => Some(FieldValue::Text(text.to_owned())),
            FieldType::Number => Self::number(text).map(FieldValue::Number),
            FieldType::Money => money(text, default_currency),
            FieldType::Date => dates::parse_iso(text).map(FieldValue::Date),
            FieldType::Boolean => flag(text).map(FieldValue::Flag),
        }
    }

    /// The value as plain text that [`Self::parse`] reads back: `1234.50 EUR`, `2026-12-01`, `true`.
    #[must_use]
    pub fn format(value: &FieldValue) -> String {
        match value {
            FieldValue::Text(text) => text.clone(),
            FieldValue::Number(number) => number.to_plain_string(),
            FieldValue::Money { amount, currency } => format!("{} {currency}", amount.to_plain_string()),
            FieldValue::Date(date) => dates::iso(*date),
            FieldValue::Flag(flag) => flag.to_string(),
        }
    }

    /// A plain number; both `1234.5` and `1.234,5` are understood, with the last separator as decimal point.
    #[must_use]
    pub fn number(raw: &str) -> Option<Decimal> {
        let cleaned = text::trim(raw).replace(['\'', ' '], "");
        let last_dot = cleaned.rfind('.');
        let last_comma = cleaned.rfind(',');
        let decimal = match (last_dot, last_comma) {
            (Some(dot), Some(comma)) => {
                if dot > comma {
                    '.'
                } else {
                    ','
                }
            }
            (_, Some(_)) if cleaned.matches(',').count() == 1 => ',',
            _ => '.',
        };
        let grouping = if decimal == '.' { ',' } else { '.' };
        let normalized = cleaned.replace(grouping, "").replace(decimal, ".");
        Decimal::parse(&normalized)
    }
}

/// `^([A-Za-z]{3})?\s*([-+]?[0-9][0-9.,']*)\s*([A-Za-z]{3})?$`, then the currency and the amount checked.
fn money(raw: &str, default_currency: Option<&CurrencyCode>) -> Option<FieldValue> {
    let three_letters = |part: &str| part.len() == 3 && part.bytes().all(|byte| byte.is_ascii_alphabetic());
    let (before, rest) = match raw.get(..3) {
        Some(prefix) if three_letters(prefix) => (prefix, &raw[3..]),
        _ => ("", raw),
    };
    let rest = rest.trim_start_matches(text::is_regex_space);
    let amount_length = amount_prefix_length(rest)?;
    let (amount, tail) = rest.split_at(amount_length);
    let after = tail.trim_start_matches(text::is_regex_space);
    if !after.is_empty() && !three_letters(after) {
        return None;
    }
    if !before.is_empty() && !after.is_empty() {
        return None;
    }
    let code = if before.is_empty() { after } else { before };
    let currency = CurrencyCode::parse(code).or_else(|| default_currency.cloned())?;
    let amount = FieldValues::number(amount)?;
    Some(FieldValue::Money { amount, currency })
}

/// The length of `[-+]?[0-9][0-9.,']*` at the start of `text`, or `None` if it does not start with one.
fn amount_prefix_length(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let sign = usize::from(matches!(bytes.first(), Some(b'-' | b'+')));
    if !bytes.get(sign).is_some_and(u8::is_ascii_digit) {
        return None;
    }
    let body = bytes[sign..].iter().take_while(|byte| matches!(byte, b'0'..=b'9' | b'.' | b',' | b'\'')).count();
    Some(sign + body)
}

fn flag(raw: &str) -> Option<bool> {
    match text::lowercase(raw).as_str() {
        "true" | "yes" | "1" => Some(true),
        "false" | "no" | "0" => Some(false),
        _ => None,
    }
}
