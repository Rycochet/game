//! Finding the workspace's Rust files and counting their lines.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// The folders whose Rust files are checked, relative to the workspace.
const FOLDERS: &[&str] = &["crates", "xtask"];

/// Every Rust file that counts, by its path from the workspace with `/`
/// between folders (the same on every OS), with its lines. Test files
/// (named `tests.rs` or ending in `_tests.rs`, or anything in a `tests`
/// folder) don't count.
pub fn rust_files(root: &Path) -> BTreeMap<String, usize> {
    let mut files = BTreeMap::new();
    for folder in FOLDERS {
        visit(root, &root.join(folder), &mut files);
    }
    files
}

fn visit(root: &Path, folder: &Path, files: &mut BTreeMap<String, usize>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() {
            if name != "target" && name != "tests" {
                visit(root, &path, files);
            }
        } else if path.extension().is_some_and(|ext| ext == "rs")
            && !name.to_string_lossy().ends_with("tests.rs")
        {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let relative = path.strip_prefix(root).unwrap_or(&path);
            let key = relative
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            files.insert(key, code_lines(&text));
        }
    }
}

/// Lines in a file, leaving out its test modules: a `#[cfg(test)]` line,
/// then `mod name;` (the tests are in their own file), or `mod name {` up to
/// the `}` that rustfmt puts alone at the start of a line to close it.
pub fn code_lines(text: &str) -> usize {
    let mut count = 0;
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        let next = lines.peek().copied().unwrap_or_default();
        let module = line == "#[cfg(test)]" && next.contains("mod ");
        if module && next.ends_with(';') {
            lines.next();
        } else if module && next.ends_with('{') {
            lines.by_ref().find(|line| *line == "}");
        } else {
            count += 1;
        }
    }
    count
}
