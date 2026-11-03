//! The WebGL 2 rules: things native graphics allow but the browser's WebGL 2
//! refuses, where Bevy then quits the whole game. Nothing on native catches
//! them, so this reads the client's source for them instead.

mod layout;
mod rules;
#[cfg(test)]
mod tests;

use std::fs;
use std::path::Path;
use std::process::ExitCode;

/// Where the client's Rust code and shaders are, relative to the workspace.
const CODE: &str = "crates/client/src";
const SHADERS: &str = "crates/client/assets/shaders";

/// Fails if any uniform isn't a multiple of 16 bytes, a render target has a
/// format WebGL 2 can't draw to, or a shader uses what WebGL 2 lacks.
pub fn check(root: &Path) -> ExitCode {
    println!("==> WebGL 2 rules (uniform sizes, render targets, shader features)");
    let mut problems = Vec::new();
    let code = files(root, CODE, "rs");
    problems.extend(layout::problems(&code));
    for (path, text) in &code {
        problems.extend(rules::target_problems(path, text));
    }
    for (path, text) in files(root, SHADERS, "wesl")
        .iter()
        .chain(&files(root, SHADERS, "wgsl"))
    {
        problems.extend(rules::shader_problems(path, text));
    }
    if problems.is_empty() {
        println!("==> All fine for WebGL 2");
        return ExitCode::SUCCESS;
    }
    for problem in &problems {
        eprintln!("{problem}");
    }
    eprintln!("==> FAILED: WebGL 2 rules (see docs/context/project.md)");
    ExitCode::FAILURE
}

/// Every file under `folder` with this extension, by its path from the
/// workspace with `/` between folders, with its text. Test files are left out.
fn files(root: &Path, folder: &str, extension: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    visit(root, &root.join(folder), extension, &mut found);
    found.sort();
    found
}

fn visit(root: &Path, folder: &Path, extension: &str, found: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(root, &path, extension, found);
        } else if path.extension().is_some_and(|ext| ext == extension)
            && !entry.file_name().to_string_lossy().ends_with("tests.rs")
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
            found.push((key, text));
        }
    }
}
