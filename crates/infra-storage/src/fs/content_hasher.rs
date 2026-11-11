//! SHA-256 of file contents, computed while streaming so large files never sit in memory.

use std::fmt::Write as _;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use dms_domain::document::ContentHash;
use sha2::{Digest, Sha256};

const BUFFER_SIZE: usize = 64 * 1024;

pub fn hash(path: &Path) -> io::Result<ContentHash> {
    hash_reader(&mut File::open(path)?)
}

/// The hash of everything `input` still has.
pub fn hash_reader(input: &mut impl Read) -> io::Result<ContentHash> {
    let mut digest = Sha256::new();
    let mut buffer = vec![0; BUFFER_SIZE];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            return Ok(to_content_hash(digest));
        }
        digest.update(&buffer[..read]);
    }
}

/// Copies `source` into the new file `target`, forces it to disk and returns the hash of what was read. The
/// caller verifies the written file against it.
pub fn copy_and_hash(source: &Path, target: &Path) -> io::Result<ContentHash> {
    let mut digest = Sha256::new();
    let mut output = OpenOptions::new().write(true).create_new(true).open(target)?;
    let mut input = File::open(source)?;
    let mut buffer = vec![0; BUFFER_SIZE];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
        output.write_all(&buffer[..read])?;
    }
    output.sync_all()?;
    Ok(to_content_hash(digest))
}

/// Lowercase hex, as `HexFormat.of().formatHex`.
fn to_content_hash(digest: Sha256) -> ContentHash {
    let hex = digest.finalize().iter().fold(String::with_capacity(64), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    });
    ContentHash::new(hex).expect("invariant: a SHA-256 digest is 64 hex digits")
}
