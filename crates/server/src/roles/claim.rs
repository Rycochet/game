//! Claiming a networked server that has no Owner: a code made at start,
//! written to `owner-claim.txt` and the log, which whoever runs the server
//! types as a new character's name.

use std::fs;
use std::path::Path;

use ring::rand::{SecureRandom, SystemRandom};

/// How long a code is: 24 of 62 letters and digits (about 143 bits), and so a
/// name a character could have.
pub(super) const CODE_LENGTH: usize = 24;

const LETTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// A new code. An error if the system's secure randomness fails.
pub(super) fn new_code() -> Result<String, String> {
    let random = SystemRandom::new();
    let mut code = String::with_capacity(CODE_LENGTH);
    while code.len() < CODE_LENGTH {
        let mut byte = [0];
        random
            .fill(&mut byte)
            .map_err(|_| "the system can't provide secure random numbers".to_owned())?;
        // 248 is the last multiple of 62 a byte holds, so each letter is as likely.
        if byte[0] < 248 {
            code.push(LETTERS[usize::from(byte[0]) % LETTERS.len()] as char);
        }
    }
    Ok(code)
}

pub(super) fn file(data: &Path) -> std::path::PathBuf {
    data.join("owner-claim.txt")
}

pub(super) fn write_file(data: &Path, code: &str) -> Result<(), String> {
    let text = format!(
        "{code}\n\nMake a new character with this as its name to become this server's Owner.\n\
         It works once, and a new code is made each time the server starts until then.\n"
    );
    let path = file(data);
    fs::write(&path, text).map_err(|error| format!("can't write {}: {error}", path.display()))
}

pub(super) fn remove_file(data: &Path) {
    let _ = fs::remove_file(file(data));
}
