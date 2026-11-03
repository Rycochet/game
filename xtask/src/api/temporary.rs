//! Finding temporary `tmp.` debug API methods.

use std::fs;
use std::path::Path;

/// Debug API methods named `tmp.` in the source: for trying things on a
/// branch, so `cargo xtask test` fails until they are removed or renamed.
pub fn temporary_methods(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    look_for_temporary(&root.join("crates"), &mut found);
    found
}

pub(super) fn look_for_temporary(folder: &Path, found: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            look_for_temporary(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            for (number, line) in text.lines().enumerate() {
                if line.contains("\"tmp.") {
                    found.push(format!(
                        "{}:{}: {}",
                        path.display(),
                        number + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
}
