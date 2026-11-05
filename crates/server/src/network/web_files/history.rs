//! The builds of the web client's wasm this server has served, kept in its
//! data folder by hash (`<hash>.wasm`), with the deltas made from them to the
//! current build (`<from>-<to>.dcz`), so making them once is enough.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use web_delta::{Hash, from_hex, hex};

/// How many builds before the current one are kept, and sent deltas.
pub(super) const KEPT: usize = 3;

/// Keeps `current` (whose hash is `hash`) in `history`, with the `KEPT`
/// builds served before it, removing older ones and deltas no longer wanted.
/// Returns those kept, newest first.
pub(super) fn keep(
    history: &Path,
    current: &[u8],
    hash: &Hash,
) -> io::Result<Vec<(Hash, PathBuf)>> {
    fs::create_dir_all(history)?;
    let mine = history.join(format!("{}.wasm", hex(hash)));
    if !mine.exists() {
        write_whole(&mine, current)?;
    }

    let mut older = Vec::new();
    for (path, named, extension) in files_in(history)? {
        if extension == "wasm" && named != Some(*hash) {
            match named {
                Some(named) => older.push((fs::metadata(&path)?.modified()?, named, path)),
                None => continue,
            }
        }
    }
    older.sort_by_key(|(modified, ..)| std::cmp::Reverse(*modified));
    for (_, _, path) in older.iter().skip(KEPT) {
        fs::remove_file(path)?;
    }
    older.truncate(KEPT);

    let wanted: Vec<_> = older
        .iter()
        .map(|(_, named, _)| delta_name(named, hash))
        .collect();
    for (path, _, extension) in files_in(history)? {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if extension == "dcz" && !wanted.iter().any(|wanted| wanted == name) {
            fs::remove_file(&path)?;
        }
    }
    Ok(older
        .into_iter()
        .map(|(_, named, path)| (named, path))
        .collect())
}

/// Where the delta from the build `from` to the build `to` is kept.
pub(super) fn delta_path(history: &Path, from: &Hash, to: &Hash) -> PathBuf {
    history.join(delta_name(from, to))
}

fn delta_name(from: &Hash, to: &Hash) -> String {
    format!("{}-{}.dcz", hex(from), hex(to))
}

/// Writes a file all at once, so one cut short (by the server stopping) is
/// never mistaken for a whole one.
pub(super) fn write_whole(path: &Path, data: &[u8]) -> io::Result<()> {
    let partial = path.with_extension("partial");
    fs::write(&partial, data)?;
    fs::rename(&partial, path)
}

/// Each file in `dir`: its path, the hash its name is (if it is one), and its
/// extension.
fn files_in(dir: &Path) -> io::Result<Vec<(PathBuf, Option<Hash>, String)>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        let extension = path.extension().and_then(|extension| extension.to_str());
        let extension = extension.unwrap_or("").to_owned();
        files.push((path.clone(), from_hex(stem), extension));
    }
    Ok(files)
}

#[cfg(test)]
mod tests;
