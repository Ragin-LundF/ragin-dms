use std::fmt;

use crate::DomainError;
use crate::text;

const SEPARATOR: char = '/';

/// A path inside a library folder, independent of where that folder lives on a device.
///
/// Always `/`-separated, NFC-normalized and relative; the empty path is the library folder itself. macOS may
/// report names decomposed (NFD) while Windows, Linux and network shares use NFC, so normalizing here is what
/// makes the same file match across folders on different file systems.
///
/// Segments are not checked against [`crate::library::FileNamePolicy`]: a path describes what exists on disk,
/// and a file created elsewhere may carry a name the policy would not allow for a new file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct RelativePath(String);

impl RelativePath {
    /// The library folder itself.
    #[must_use]
    pub fn root() -> Self {
        Self(String::new())
    }

    /// Parses a `/`-separated path as stored in the catalog. A trailing separator is ignored.
    ///
    /// # Errors
    /// Fails when the path is absolute or has empty, `.` or `..` segments.
    pub fn of(raw: &str) -> Result<Self, DomainError> {
        let value = normalized(raw);
        match problem_of(&value) {
            Some(problem) => Err(DomainError::InvalidArgument(problem)),
            None => Ok(Self(value)),
        }
    }

    /// Like [`Self::of`] for a path a user typed: `None` if it is absolute or has empty, `.` or `..` segments.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        Self::of(raw).ok()
    }

    /// Builds a path from file system segments, which may contain characters such as `\`.
    ///
    /// # Errors
    /// Fails like [`Self::of`].
    pub fn from_segments<S: AsRef<str>>(segments: &[S]) -> Result<Self, DomainError> {
        let joined: Vec<&str> = segments.iter().map(AsRef::as_ref).collect();
        Self::of(&joined.join("/"))
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn segments(&self) -> Vec<&str> {
        if self.is_root() { Vec::new() } else { self.0.split(SEPARATOR).collect() }
    }

    /// The last segment, or an empty string for the root.
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.0.rsplit(SEPARATOR).next().unwrap_or_default()
    }

    /// The containing folder, or `None` for the root.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        if self.is_root() {
            return None;
        }
        Some(Self(self.0.rsplit_once(SEPARATOR).map(|(parent, _)| parent.to_owned()).unwrap_or_default()))
    }

    /// # Errors
    /// Fails when the result is no valid path, for example for a name of `..`.
    pub fn resolve(&self, name: &str) -> Result<Self, DomainError> {
        if self.is_root() { Self::of(name) } else { Self::of(&format!("{}{SEPARATOR}{name}", self.0)) }
    }

    /// `true` if this path lies strictly below `folder`.
    #[must_use]
    pub fn is_inside(&self, folder: &Self) -> bool {
        if self == folder {
            return false;
        }
        folder.is_root() || (self.0.starts_with(&folder.0) && self.0[folder.0.len()..].starts_with(SEPARATOR))
    }

    /// Moves this path from below `from` to below `to`, keeping the part after `from`.
    ///
    /// # Errors
    /// Fails when this path is neither `from` nor inside it.
    pub fn relocate(&self, from: &Self, to: &Self) -> Result<Self, DomainError> {
        if self == from {
            return Ok(to.clone());
        }
        if !self.is_inside(from) {
            return Err(DomainError::InvalidArgument(format!("{} is not inside {}", self.0, from.0)));
        }
        let remainder = if from.is_root() { self.0.as_str() } else { &self.0[from.0.len() + 1..] };
        to.resolve(remainder)
    }
}

impl fmt::Display for RelativePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

fn normalized(raw: &str) -> String {
    text::nfc(raw.strip_suffix(SEPARATOR).unwrap_or(raw))
}

fn problem_of(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let drive_prefix = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if value.starts_with(SEPARATOR) || drive_prefix {
        return Some(format!("Path must be relative: {value}"));
    }
    if !value.is_empty() && value.split(SEPARATOR).any(|segment| matches!(segment, "" | "." | "..")) {
        return Some(format!("Path must not contain empty, '.' or '..' segments: {value}"));
    }
    None
}
