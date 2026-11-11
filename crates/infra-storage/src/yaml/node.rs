use indexmap::IndexMap;

/// A parsed YAML document as Jackson sees it with its default JSON schema: plain scalars resolve to null,
/// booleans and numbers only in their JSON spelling, everything else (also `~`, `yes`, `0123`, dates) is text.
/// Numbers keep their source text, so a value of a newer version is written back as it was read.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Null,
    Bool(bool),
    Int(String),
    Float(String),
    Text(String),
    Seq(Vec<Node>),
    Map(IndexMap<String, Node>),
}

impl Node {
    /// The node for a plain (unquoted) scalar, resolved like Jackson's `JsonScalarResolver`.
    pub fn plain(text: &str) -> Self {
        match text {
            "" | "null" => Self::Null,
            "true" => Self::Bool(true),
            "false" => Self::Bool(false),
            _ if is_json_int(text) => Self::Int(text.to_owned()),
            _ if is_json_float(text) => Self::Float(text.to_owned()),
            _ => Self::Text(text.to_owned()),
        }
    }

    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub fn int(value: impl Into<i64>) -> Self {
        Self::Int(value.into().to_string())
    }

    /// Jackson writes a `Double` with Java's `Double.toString`.
    pub fn float(value: f64) -> Self {
        Self::Float(value.to_string())
    }

    /// The text of a scalar, as Jackson coerces any scalar into a `String` property; `None` for null.
    pub fn scalar_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) | Self::Int(text) | Self::Float(text) => Some(text),
            Self::Bool(true) => Some("true"),
            Self::Bool(false) => Some("false"),
            Self::Null | Self::Seq(_) | Self::Map(_) => None,
        }
    }
}

/// `-?(0|[1-9][0-9]*)`
fn is_json_int(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    match digits.as_bytes() {
        [b'0'] => true,
        [first, rest @ ..] => (b'1'..=b'9').contains(first) && rest.iter().all(u8::is_ascii_digit),
        [] => false,
    }
}

/// `-?(0|[1-9][0-9]*)(\.[0-9]*)?([eE][-+]?[0-9]+)?`, `-?.inf` or `.nan`
fn is_json_float(text: &str) -> bool {
    if matches!(text, ".inf" | "-.inf" | ".nan") {
        return true;
    }
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
        Some(index) => (&unsigned[..index], Some(&unsigned[index + 1..])),
        None => (unsigned, None),
    };
    let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let integer_ok = is_json_int(integer) && !integer.starts_with('-');
    let fraction_ok = fraction.bytes().all(|byte| byte.is_ascii_digit());
    let exponent_ok = exponent.is_none_or(|exponent| {
        let digits = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    });
    integer_ok && fraction_ok && exponent_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_scalars_resolve_like_jacksons_json_schema() {
        assert_eq!(Node::plain("null"), Node::Null);
        assert_eq!(Node::plain(""), Node::Null);
        assert_eq!(Node::plain("true"), Node::Bool(true));
        assert_eq!(Node::plain("5"), Node::Int("5".to_owned()));
        assert_eq!(Node::plain("-0"), Node::Int("-0".to_owned()));
        assert_eq!(Node::plain("1.50"), Node::Float("1.50".to_owned()));
        assert_eq!(Node::plain("1e3"), Node::Float("1e3".to_owned()));
        assert_eq!(Node::plain("-.inf"), Node::Float("-.inf".to_owned()));
        for text in ["~", "Null", "True", "yes", "0123", "+5", ".5", "0x1F", "2024-01-01", "1:30", "1e", "-"] {
            assert_eq!(Node::plain(text), Node::text(text), "{text}");
        }
    }
}
