//! `large-files.txt`: the files allowed over the limit, and how far.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

/// Where the list is kept, relative to the workspace.
pub const FILE: &str = "xtask/large-files.txt";

/// What the list starts with, rewritten each time it is saved.
const HEADER: &str = "\
# Rust files allowed over the file size limit (see \"File size\" in CLAUDE.md).
# Each line: path, the most lines it may have (tests not counted), and an
# optional `# reason` for keeping it large (such as a long data table).
# Files without a reason are waiting to be split. A listed file may only
# shrink: `cargo xtask sizes` lowers the numbers and removes files back under
# the limit. Only add a file with the owner's agreement.
";

/// One listed file.
pub struct Entry {
    /// The most lines it may have.
    pub lines: usize,
    /// Why it is kept large, with its `#`, or empty.
    pub reason: String,
}

/// The listed files by path. A missing or unreadable list is empty, and
/// lines that don't parse are skipped.
pub fn read(root: &Path) -> BTreeMap<String, Entry> {
    let text = fs::read_to_string(root.join(FILE)).unwrap_or_default();
    text.lines().filter_map(parse).collect()
}

/// Parses `path lines # reason`.
pub fn parse(line: &str) -> Option<(String, Entry)> {
    let (entry, reason) = match line.find('#') {
        Some(at) => (&line[..at], line[at..].trim()),
        None => (line, ""),
    };
    let mut words = entry.split_whitespace();
    let path = words.next()?.to_string();
    let lines = words.next()?.parse().ok()?;
    let reason = reason.to_string();
    Some((path, Entry { lines, reason }))
}

/// Saves the list, sorted by path.
pub fn write(root: &Path, entries: &BTreeMap<String, Entry>) -> io::Result<()> {
    let mut text = HEADER.to_string();
    for (path, entry) in entries {
        text.push_str(&format!("{path} {}", entry.lines));
        if !entry.reason.is_empty() {
            text.push_str(&format!(" {}", entry.reason));
        }
        text.push('\n');
    }
    fs::write(root.join(FILE), text)
}
