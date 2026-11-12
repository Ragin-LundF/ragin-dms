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

    /// Zero with scale 0.
    #[must_use]
    pub fn zero() -> Self {
        Self { unscaled: BigInt::from(0), scale: 0 }
    }

    /// Like `BigDecimal.add`: exact, with the larger of both scales.
    #[must_use]
    pub fn plus(&self, other: &Self) -> Self {
        let scale = self.scale.max(other.scale);
        Self { unscaled: self.aligned_to(scale) + other.aligned_to(scale), scale }
    }

    /// Like `BigDecimal.negate`: same scale.
    #[must_use]
    pub fn negated(&self) -> Self {
        Self { unscaled: -&self.unscaled, scale: self.scale }
    }

    /// Like `BigDecimal.multiply` by a whole number: exact, same scale.
    #[must_use]
    pub fn times(&self, factor: u32) -> Self {
        Self { unscaled: &self.unscaled * BigInt::from(factor), scale: self.scale }
    }

    /// `self / divisor` with `scale` decimal places, rounded half up (away from zero), like
    /// `BigDecimal.divide(divisor, scale, RoundingMode.HALF_UP)`.
    ///
    /// # Panics
    /// Never for a divisor above zero; a zero divisor is a programming error.
    #[must_use]
    pub fn divided_rounded(&self, divisor: u32, scale: u32) -> Self {
        assert!(divisor > 0, "invariant: division by zero");
        let target = i32::try_from(scale).unwrap_or(i32::MAX);
        let (numerator, denominator) = if target >= self.scale {
            (shifted(&self.unscaled, target.abs_diff(self.scale)), BigInt::from(divisor))
        } else {
            (self.unscaled.clone(), shifted(&BigInt::from(divisor), self.scale.abs_diff(target)))
        };
        let quotient = &numerator / &denominator;
        let remainder = &numerator % &denominator;
        let doubled = BigInt::from(remainder.magnitude().clone()) * 2;
        let rounded = if doubled >= denominator {
            if numerator.sign() == Sign::Minus { quotient - 1 } else { quotient + 1 }
        } else {
            quotient
        };
        Self { unscaled: rounded, scale: target }
    }

    fn aligned_to(&self, scale: i32) -> BigInt {
        shifted(&self.unscaled, scale.abs_diff(self.scale))
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
    #[test]
    fn arithmetic_is_exact_and_rounds_half_up() {
        assert_eq!(decimal("12.5").plus(&decimal("0.25")), decimal("12.75"));
        assert_eq!(decimal("-1").plus(&decimal("0.50")), decimal("-0.50"));
        assert_eq!(Decimal::zero().plus(&decimal("3.10")), decimal("3.10"));
        assert_eq!(decimal("29.99").times(12), decimal("359.88"));
        assert_eq!(decimal("100").divided_rounded(3, 2), decimal("33.33"));
        assert_eq!(decimal("0.05").divided_rounded(2, 2), decimal("0.03"));
        assert_eq!(decimal("-0.05").divided_rounded(2, 2), decimal("-0.03"));
        assert_eq!(decimal("1.2345").divided_rounded(1, 2), decimal("1.23"));
        assert_eq!(decimal("1.235").divided_rounded(1, 2), decimal("1.24"));
        assert_eq!(decimal("1e2").divided_rounded(4, 2), decimal("25.00"));
    }
}
