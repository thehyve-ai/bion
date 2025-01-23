use std::{fs, io, path::Path};

use eth2_keystore::PlainText;
use rand::{distributions::Alphanumeric, Rng};

use super::ZeroizeString;

/// The minimum number of characters required for a wallet password.
pub const MINIMUM_PASSWORD_LEN: usize = 12;
/// The `Alphanumeric` crate only generates a-z, A-Z, 0-9, therefore it has a range of 62
/// characters.
///
/// 62**48 is greater than 255**32, therefore this password has more bits of entropy than a byte
/// array of length 32.
const DEFAULT_PASSWORD_LEN: usize = 48;

/// Reads a password file into a `ZeroizeString` struct, with new-lines removed.
pub fn read_password_string<P: AsRef<Path>>(path: P) -> Result<ZeroizeString, String> {
    fs::read(path)
        .map_err(|e| format!("Error opening file: {:?}", e))
        .map(strip_off_newlines)
        .and_then(|bytes| {
            String::from_utf8(bytes)
                .map_err(|e| format!("Error decoding utf8: {:?}", e))
                .map(Into::into)
        })
}

/// Remove any number of newline or carriage returns from the end of a vector of bytes.
pub fn strip_off_newlines(mut bytes: Vec<u8>) -> Vec<u8> {
    let mut strip_off = 0;
    for (i, byte) in bytes.iter().rev().enumerate() {
        if *byte == b'\n' || *byte == b'\r' {
            strip_off = i + 1;
        } else {
            break;
        }
    }
    bytes.truncate(bytes.len() - strip_off);
    bytes
}

/// Reads a password file into a Zeroize-ing `PlainText` struct, with new-lines removed.
pub fn read_password<P: AsRef<Path>>(path: P) -> Result<PlainText, io::Error> {
    fs::read(path).map(strip_off_newlines).map(Into::into)
}

/// Generates a random alphanumeric password of length `DEFAULT_PASSWORD_LEN` as `ZeroizeString`.
pub fn random_password_string() -> ZeroizeString {
    random_password_raw_string().into()
}

/// Common implementation for `random_password` and `random_password_string`.
fn random_password_raw_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(DEFAULT_PASSWORD_LEN)
        .map(char::from)
        .collect()
}
