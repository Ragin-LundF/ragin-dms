//! Java's `DataOutputStream` and `DataInputStream`: big-endian numbers and `writeUTF` strings (a two-byte length,
//! then "modified UTF-8": every UTF-16 unit on its own, `U+0000` as two bytes).

use std::io;

#[derive(Debug, Default)]
pub struct DataOutput {
    bytes: Vec<u8>,
}

impl DataOutput {
    pub fn write_int(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    /// `Float.floatToIntBits`: every NaN is written as the canonical NaN.
    pub fn write_float(&mut self, value: f32) {
        let bits = if value.is_nan() { 0x7fc0_0000 } else { value.to_bits() };
        self.bytes.extend_from_slice(&bits.to_be_bytes());
    }

    /// # Errors
    /// Fails for a string longer than 65535 encoded bytes, like Java's `UTFDataFormatException`.
    pub fn write_utf(&mut self, text: &str) -> io::Result<()> {
        let mut encoded = Vec::with_capacity(text.len());
        for unit in text.encode_utf16() {
            match unit {
                0x0001..=0x007F => encoded.extend(unit.to_be_bytes().get(1)),
                0x0000 | 0x0080..=0x07FF => {
                    encoded.push(0xC0 | low_byte(unit >> 6));
                    encoded.push(0x80 | low_byte(unit & 0x3F));
                }
                _ => {
                    encoded.push(0xE0 | low_byte(unit >> 12));
                    encoded.push(0x80 | low_byte((unit >> 6) & 0x3F));
                    encoded.push(0x80 | low_byte(unit & 0x3F));
                }
            }
        }
        let length = u16::try_from(encoded.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "encoded string too long"))?;
        self.bytes.extend_from_slice(&length.to_be_bytes());
        self.bytes.extend_from_slice(&encoded);
        Ok(())
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

fn low_byte(value: u16) -> u8 {
    value.to_be_bytes()[1]
}

/// Reads what [`DataOutput`] wrote; `None` at the end of the data or for malformed text.
#[derive(Debug)]
pub struct DataInput<'a> {
    bytes: &'a [u8],
}

impl<'a> DataInput<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn remaining(&self) -> usize {
        self.bytes.len()
    }

    pub fn read_int(&mut self) -> Option<i32> {
        Some(i32::from_be_bytes(self.take(4)?.try_into().ok()?))
    }

    pub fn read_float(&mut self) -> Option<f32> {
        Some(f32::from_bits(u32::from_be_bytes(self.take(4)?.try_into().ok()?)))
    }

    pub fn read_utf(&mut self) -> Option<String> {
        let length = u16::from_be_bytes(self.take(2)?.try_into().ok()?);
        let encoded = self.take(usize::from(length))?;
        let mut units = Vec::with_capacity(encoded.len());
        let mut index = 0;
        while index < encoded.len() {
            let first = u16::from(encoded[index]);
            let continuation = |offset: usize| {
                encoded
                    .get(index + offset)
                    .copied()
                    .filter(|byte| byte & 0xC0 == 0x80)
                    .map(|byte| u16::from(byte & 0x3F))
            };
            let (unit, width) = match first >> 4 {
                0..=7 => (first, 1),
                12 | 13 => (((first & 0x1F) << 6) | continuation(1)?, 2),
                14 => (((first & 0x0F) << 12) | (continuation(1)? << 6) | continuation(2)?, 3),
                _ => return None,
            };
            units.push(unit);
            index += width;
        }
        String::from_utf16(&units).ok()
    }

    fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        if count > self.bytes.len() {
            return None;
        }
        let (taken, rest) = self.bytes.split_at(count);
        self.bytes = rest;
        Some(taken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_are_written_as_modified_utf8() {
        let mut output = DataOutput::default();
        output.write_utf("a\u{0}ä€😀").expect("short string");

        let bytes = output.into_bytes();
        let expected = [0, 14, b'a', 0xC0, 0x80, 0xC3, 0xA4, 0xE2, 0x82, 0xAC, 0xED, 0xA0, 0xBD, 0xED, 0xB8, 0x80];
        assert_eq!(bytes, expected);
        assert_eq!(DataInput::new(&bytes).read_utf().as_deref(), Some("a\u{0}ä€😀"));
    }

    #[test]
    fn truncated_or_malformed_data_reads_as_none() {
        assert_eq!(DataInput::new(&[0, 3, b'a']).read_utf(), None);
        assert_eq!(DataInput::new(&[0, 1, 0xC3]).read_utf(), None);
        assert_eq!(DataInput::new(&[0, 0, 0]).read_int(), None);
        let too_long = "x".repeat(70_000);
        assert!(DataOutput::default().write_utf(&too_long).is_err());
    }
}
