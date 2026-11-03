//! The file size rule: no Rust file over `LIMIT` lines, not counting its
//! tests, unless it is listed in `large-files.txt`, and a listed file may only
//! shrink. See "File size" in CLAUDE.md.

mod count;
mod list;
#[cfg(test)]
mod tests;

use std::path::Path;
use std::process::ExitCode;

/// Most lines a file may have, leaving out its tests.
pub const LIMIT: usize = 300;

/// Fails if a file is over the limit and not listed, a listed file has grown
/// past its entry, or an entry is no longer needed.
pub fn check(root: &Path) -> ExitCode {
    println!("==> File sizes (at most {LIMIT} lines, tests not counted)");
    let files = count::rust_files(root);
    let listed = list::read(root);
    let mut problems = Vec::new();
    let mut shrunk = 0;
    for (path, &lines) in &files {
        match listed.get(path) {
            None if lines > LIMIT => problems.push(format!(
                "{path}: {lines} lines: split it (one idea per file), or ask the owner to list it in {}",
                list::FILE
            )),
            Some(entry) if lines > entry.lines => problems.push(format!(
                "{path}: {lines} lines, grown past its {} in {}: split it rather than growing it",
                entry.lines,
                list::FILE
            )),
            Some(_) if lines <= LIMIT => problems.push(format!(
                "{path}: {lines} lines, no longer over the limit: run `cargo xtask sizes`"
            )),
            Some(entry) if lines < entry.lines => shrunk += 1,
            _ => {}
        }
    }
    for path in listed.keys().filter(|path| !files.contains_key(*path)) {
        problems.push(format!("{path}: listed but gone: run `cargo xtask sizes`"));
    }

    if shrunk > 0 {
        println!("==> {shrunk} listed files have shrunk: `cargo xtask sizes` lowers their entries");
    }
    if problems.is_empty() {
        println!("==> All within their sizes");
        return ExitCode::SUCCESS;
    }
    for problem in &problems {
        eprintln!("{problem}");
    }
    eprintln!("==> FAILED: file sizes");
    ExitCode::FAILURE
}

/// Lowers each entry to its file's size now, and drops the entries of files
/// that are gone or back under the limit. Never adds or raises an entry.
pub fn update(root: &Path) -> ExitCode {
    let files = count::rust_files(root);
    let mut kept = list::read(root);
    let before = kept.len();
    kept.retain(|path, _| files.get(path).is_some_and(|&lines| lines > LIMIT));
    for (path, entry) in kept.iter_mut() {
        entry.lines = entry.lines.min(files[path]);
    }
    match list::write(root, &kept) {
        Ok(()) => {
            println!(
                "==> {}: {} entries, {} removed",
                list::FILE,
                kept.len(),
                before - kept.len()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("==> FAILED: writing {}: {error}", list::FILE);
            ExitCode::FAILURE
        }
    }
}
