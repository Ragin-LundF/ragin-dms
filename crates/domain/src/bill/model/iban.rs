use std::fmt;

use crate::DomainError;

/// An International Bank Account Number that passed the ISO 13616 mod-97 check, stored without blanks and in upper
/// case (`DE89370400440532013000`).
///
/// shortcut: checks the general format and the checksum, not the length per country; add the ISO 13616 length
/// table if wrong-length IBANs with a valid checksum ever show up.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Iban(String);

impl Iban {
    /// Reads an IBAN as people write it: blanks anywhere, any case.
    ///
    /// # Errors
    /// Fails when the text is no IBAN or its checksum is wrong.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let compact: String =
            raw.chars().filter(|character| !character.is_whitespace()).collect::<String>().to_uppercase();
        if !has_iban_shape(&compact) {
            return Err(DomainError::InvalidArgument(format!("Not an IBAN: {raw}")));
        }
        if checksum(&compact) != 1 {
            return Err(DomainError::InvalidArgument(format!("The IBAN checksum is wrong: {raw}")));
        }
        Ok(Self(compact))
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }

    /// The IBAN in groups of four, as printed on paper: `DE89 3704 0044 0532 0130 00`.
    #[must_use]
    pub fn grouped(&self) -> String {
        let characters: Vec<char> = self.0.chars().collect();
        characters.chunks(4).map(|group| group.iter().collect::<String>()).collect::<Vec<_>>().join(" ")
    }
}

impl fmt::Display for Iban {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Country code, two check digits, then 11 to 30 letters or digits.
fn has_iban_shape(compact: &str) -> bool {
    let bytes = compact.as_bytes();
    (15..=34).contains(&bytes.len())
        && bytes[..2].iter().all(u8::is_ascii_uppercase)
        && bytes[2..4].iter().all(u8::is_ascii_digit)
        && bytes[4..].iter().all(u8::is_ascii_alphanumeric)
}

/// The remainder modulo 97 of the number formed by moving the first four characters to the end and replacing each
/// letter by 10..35.
fn checksum(compact: &str) -> u32 {
    let (head, tail) = compact.split_at(4);
    tail.chars().chain(head.chars()).fold(0, |remainder, character| {
        let value = character.to_digit(36).unwrap_or(0);
        if value >= 10 { (remainder * 100 + value) % 97 } else { (remainder * 10 + value) % 97 }
    })
}
