use std::cmp::Ordering;
use std::fmt;

use num_bigint::{BigInt, Sign};

/// An arbitrary-precision decimal number with a scale, standing in for `java.math.BigDecimal`.
///
/// Like `BigDecimal`, equality includes the scale (`1.0` and `1.00` are not equal, as in the Kotlin data
/// classes that hold them), while [`Self::compare_value`] compares the numbers only. The value is
/// `unscaled × 10^-scale`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Decimal {
    unscaled: BigInt,
    scale: i32,
}

impl Decimal {
    /// Parses text the way `BigDecimal(String)` does: an optional sign, digits with an optional decimal
    /// point, and an optional exponent such as `e-3`. No blanks, no grouping.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (mantissa, exponent) = match text.find(['e', 'E']) {
            Some(index) => (&text[..index], parse_exponent(&text[index + 1..])?),
            None => (text, 0),
        };
        let (negative, unsigned) = match mantissa.as_bytes().first()? {
            b'-' => (true, &mantissa[1..]),
            b'+' => (false, &mantissa[1..]),
            _ => (false, mantissa),
        };
        let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let all_digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
        if integer.len() + fraction.len() == 0 || !all_digits(integer) || !all_digits(fraction) {
            return None;
        }
        let magnitude: BigInt = format!("{integer}{fraction}").parse().ok()?;
        let scale = i32::try_from(i64::try_from(fraction.len()).ok()? - exponent).ok()?;
        Some(Self { unscaled: if negative { -magnitude } else { magnitude }, scale })
    }

    /// Like `BigDecimal.toPlainString()`: no exponent, trailing zeros of the scale kept.
    #[must_use]
    pub fn to_plain_string(&self) -> String {
        if self.scale <= 0 {
            if self.unscaled.sign() == Sign::NoSign {
                return "0".to_owned();
            }
            return format!("{}{}", self.unscaled, "0".repeat(self.scale.unsigned_abs() as usize));
        }
        let digits = self.unscaled.magnitude().to_string();
        let sign = if self.unscaled.sign() == Sign::Minus { "-" } else { "" };
        let scale = self.scale.unsigned_abs() as usize;
        if digits.len() > scale {
            let (integer, fraction) = digits.split_at(digits.len() - scale);
            format!("{sign}{integer}.{fraction}")
        } else {
            format!("{sign}0.{}{digits}", "0".repeat(scale - digits.len()))
        }
    }

    /// Like `BigDecimal.compareTo`: by value, ignoring the scale.
    #[must_use]
    pub fn compare_value(&self, other: &Self) -> Ordering {
        let signs = signum(&self.unscaled).cmp(&signum(&other.unscaled));
        if signs != Ordering::Equal || self.unscaled.sign() == Sign::NoSign {
            return signs;
        }
        // Different orders of magnitude decide without aligning, which could need huge powers of ten.
        let magnitudes = self.adjusted_exponent().cmp(&other.adjusted_exponent());
        if magnitudes != Ordering::Equal {
            return if self.unscaled.sign() == Sign::Minus { magnitudes.reverse() } else { magnitudes };
        }
        let (left, right) = if self.scale >= other.scale {
            (self.unscaled.clone(), shifted(&other.unscaled, self.scale.abs_diff(other.scale)))
        } else {
            (shifted(&self.unscaled, other.scale.abs_diff(self.scale)), other.unscaled.clone())
        };
        left.cmp(&right)
    }

    /// Like `BigDecimal.longValue()`: the integer part, truncated towards zero, in its low 64 bits.
    #[must_use]
    pub fn to_i64_truncated(&self) -> i64 {
        let integer = if self.scale <= 0 {
            let zeros = self.scale.unsigned_abs();
            // 10^64 is a multiple of 2^64, so the low 64 bits of a bigger shift are zero.
            if zeros >= 64 {
                return 0;
            }
            shifted(&self.unscaled, zeros)
        } else {
            let scale = self.scale.unsigned_abs();
            if u64::from(scale) > digit_count(&self.unscaled) {
                return 0;
            }
            &self.unscaled / BigInt::from(10u32).pow(scale)
        };
        low_64_bits(&integer)
    }

    /// The order of magnitude: digits minus scale.
    fn adjusted_exponent(&self) -> i64 {
        i64::try_from(digit_count(&self.unscaled)).unwrap_or(i64::MAX) - i64::from(self.scale)
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_plain_string())
    }
}

fn parse_exponent(text: &str) -> Option<i64> {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn signum(value: &BigInt) -> i8 {
    match value.sign() {
        Sign::Minus => -1,
        Sign::NoSign => 0,
        Sign::Plus => 1,
    }
}

fn digit_count(value: &BigInt) -> u64 {
    value.magnitude().to_string().len() as u64
}

fn shifted(value: &BigInt, zeros: u32) -> BigInt {
    value * BigInt::from(10u32).pow(zeros)
}

fn low_64_bits(value: &BigInt) -> i64 {
    let bytes = value.to_signed_bytes_le();
    let fill = if value.sign() == Sign::Minus { 0xFF } else { 0 };
    let mut low = [fill; 8];
    for (target, byte) in low.iter_mut().zip(bytes) {
        *target = byte;
    }
    i64::from_le_bytes(low)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimal(text: &str) -> Decimal {
        Decimal::parse(text).expect("valid decimal")
    }

    #[test]
    fn parses_like_big_decimal() {
        assert_eq!(decimal("120.50").to_plain_string(), "120.50");
        assert_eq!(decimal("-3.25").to_plain_string(), "-3.25");
        assert_eq!(decimal(".5").to_plain_string(), "0.5");
        assert_eq!(decimal("5.").to_plain_string(), "5");
        assert_eq!(decimal("1e3").to_plain_string(), "1000");
        assert_eq!(decimal("1.5E-3").to_plain_string(), "0.0015");
        assert_eq!(decimal("-0.00").to_plain_string(), "0.00");
        assert_eq!(decimal("0e5").to_plain_string(), "0");
        for bad in ["", "+", ".", "1,5", " 1", "1e", "e5", "1.2.3", "twelve", "1e99999999999"] {
            assert_eq!(Decimal::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn equality_sees_the_scale_but_comparison_does_not() {
        assert_ne!(decimal("1.0"), decimal("1.00"));
        assert_eq!(decimal("1.0").compare_value(&decimal("1.00")), Ordering::Equal);
        assert_eq!(decimal("250").compare_value(&decimal("100")), Ordering::Greater);
        assert_eq!(decimal("-250").compare_value(&decimal("-100")), Ordering::Less);
        assert_eq!(decimal("0.001").compare_value(&decimal("1e-999999")), Ordering::Greater);
    }

    #[test]
    fn long_value_truncates_towards_zero() {
        assert_eq!(decimal("30").to_i64_truncated(), 30);
        assert_eq!(decimal("-7.9").to_i64_truncated(), -7);
        assert_eq!(decimal("0.5").to_i64_truncated(), 0);
        assert_eq!(decimal("1e100").to_i64_truncated(), 0);
    }
}
