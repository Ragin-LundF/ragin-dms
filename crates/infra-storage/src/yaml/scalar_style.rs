//! The style Jackson 3 (`YAMLGenerator` with `MINIMIZE_QUOTES`, `LITERAL_BLOCK_STYLE` and
//! `ALWAYS_QUOTE_NUMBERS_AS_STRINGS`) asks the emitter for, and how it prints numbers.

/// The style requested for a scalar; the emitter may still fall back to a quoted style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestedStyle {
    Plain,
    DoubleQuoted,
    Literal,
}

/// `YAMLGenerator.writeString`.
pub fn for_value(text: &str) -> RequestedStyle {
    if text.is_empty() {
        RequestedStyle::DoubleQuoted
    } else if text.contains('\n') {
        RequestedStyle::Literal
    } else if is_reserved_keyword(text) || has_quotable_char(text) || looks_like_number(text) {
        RequestedStyle::DoubleQuoted
    } else {
        RequestedStyle::Plain
    }
}

/// `YAMLGenerator.writeName` with `StringQuotingChecker.Default.needToQuoteName`.
pub fn for_key(name: &str) -> RequestedStyle {
    let starts_like_number = name.starts_with(|first: char| first.is_ascii_digit() || matches!(first, '-' | '+' | '.'));
    if is_reserved_keyword(name) || starts_like_number || name.chars().any(|character| u32::from(character) < 0x20) {
        RequestedStyle::DoubleQuoted
    } else {
        RequestedStyle::Plain
    }
}

/// Java's `Double.toString`, which Jackson uses for a `Double`; `.inf`, `-.inf` and `.nan` for the special values.
pub fn java_double(value: f64) -> String {
    if value.is_nan() {
        return ".nan".to_owned();
    }
    if value.is_infinite() {
        return if value > 0.0 { ".inf" } else { "-.inf" }.to_owned();
    }
    if value == 0.0 {
        return if value.is_sign_negative() { "-0.0" } else { "0.0" }.to_owned();
    }
    // Both print the shortest digits that read back as the same double; only the layout differs.
    let scientific = format!("{:e}", value.abs());
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let sign = if value < 0.0 { "-" } else { "" };
    if (-3..7).contains(&exponent) {
        let point = exponent + 1;
        let text = if point <= 0 {
            format!("0.{}{digits}", "0".repeat(point.unsigned_abs() as usize))
        } else {
            let point = point.unsigned_abs() as usize;
            let padded = format!("{digits:0<point$}");
            let (integer, fraction) = padded.split_at(point);
            format!("{integer}.{}", if fraction.is_empty() { "0" } else { fraction })
        };
        format!("{sign}{text}")
    } else {
        let (first, rest) = digits.split_at(1);
        format!("{sign}{first}.{}E{exponent}", if rest.is_empty() { "0" } else { rest })
    }
}

/// `StringQuotingChecker.isReservedKeyword`.
fn is_reserved_keyword(text: &str) -> bool {
    text.is_empty()
        || text.starts_with('~')
        || matches!(
            text,
            "false"
                | "False"
                | "FALSE"
                | "n"
                | "N"
                | "no"
                | "No"
                | "NO"
                | "null"
                | "Null"
                | "NULL"
                | "on"
                | "On"
                | "ON"
                | "off"
                | "Off"
                | "OFF"
                | "true"
                | "True"
                | "TRUE"
                | "y"
                | "Y"
                | "yes"
                | "Yes"
                | "YES"
        )
}

/// `StringQuotingChecker.valueHasQuotableChar`.
fn has_quotable_char(text: &str) -> bool {
    let units: Vec<u16> = text.encode_utf16().collect();
    let blank = |unit: u16| unit == u16::from(b' ') || unit == u16::from(b'\t');
    units.iter().enumerate().any(|(index, &unit)| match u8::try_from(unit) {
        Ok(b'[' | b']' | b'{' | b'}' | b',') => true,
        Ok(b'#') => index == 0 || blank(units[index - 1]),
        Ok(b':') => index + 1 == units.len() || blank(units[index + 1]),
        _ => false,
    })
}

/// `YAMLGenerator._looksLikeYAMLNumber`: Jackson's plain-number pattern or the JSON schema's int and float.
fn looks_like_number(text: &str) -> bool {
    if !text.starts_with(|first: char| first.is_ascii_digit() || matches!(first, '+' | '-' | '.')) {
        return false;
    }
    is_plain_number(text) || !matches!(crate::yaml::Node::plain(text), crate::yaml::Node::Text(_))
}

/// `[+-]?[0-9]*(\.[0-9]*)?`
fn is_plain_number(text: &str) -> bool {
    let unsigned = text.strip_prefix(['+', '-']).unwrap_or(text);
    let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    integer.bytes().all(|byte| byte.is_ascii_digit()) && fraction.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doubles_print_like_java() {
        assert_eq!(java_double(0.75), "0.75");
        assert_eq!(java_double(1.0), "1.0");
        assert_eq!(java_double(1.0e-4), "1.0E-4");
        assert_eq!(java_double(0.001), "0.001");
        assert_eq!(java_double(1.0e7), "1.0E7");
        assert_eq!(java_double(9_999_999.0), "9999999.0");
        assert_eq!(java_double(123_456.789), "123456.789");
        assert_eq!(java_double(-2.5e-10), "-2.5E-10");
        assert_eq!(java_double(f64::INFINITY), ".inf");
    }
}
