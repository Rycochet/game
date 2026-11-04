//! Where cargo puts what it builds: `target/` in the workspace, unless a cargo
//! setting (`build.target-dir`) or `CARGO_TARGET_DIR` moves it elsewhere.

#[cfg(test)]
mod tests;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The target folder as cargo reports it, or `target/` in `root` if cargo
/// can't be asked.
pub fn target_dir(root: &Path) -> PathBuf {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| from_metadata(&String::from_utf8_lossy(&output.stdout)))
        .unwrap_or_else(|| root.join("target"))
}

/// Reads `"target_directory":"..."` out of `cargo metadata`'s JSON. It's the
/// only value needed, so this finds it by hand rather than adding a JSON
/// library. JSON writes each `\` in a Windows path as `\\`.
fn from_metadata(json: &str) -> Option<PathBuf> {
    let key = "\"target_directory\":\"";
    let start = json.find(key)? + key.len();
    let end = start + json[start..].find('"')?;
    Some(PathBuf::from(json[start..end].replace("\\\\", "\\")))
}
