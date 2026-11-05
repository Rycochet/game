//! The two Dockerfiles must make the same image: `Dockerfile` compiles
//! everything inside Docker (for building it yourself), `Dockerfile.github`
//! only copies in what GitHub's Docker workflow compiled. Everything after the
//! marker line is the same in both, and this checks it is.

#[cfg(test)]
mod tests;

use std::fs;
use std::path::Path;
use std::process::ExitCode;

const LOCAL: &str = "Dockerfile";
const GITHUB: &str = "Dockerfile.github";

/// The line both files share from. What follows it must match exactly.
const MARKER: &str = "# --- Same in Dockerfile and Dockerfile.github from here to the end";

/// Fails unless both Dockerfiles are there with the same part after the marker.
pub fn check(root: &Path) -> ExitCode {
    println!("==> Dockerfiles: {LOCAL} and {GITHUB} end the same");
    let read = |name: &str| fs::read_to_string(root.join(name)).map_err(|e| format!("{name}: {e}"));
    let problem = match (read(LOCAL), read(GITHUB)) {
        (Ok(local), Ok(github)) => problem(&local, &github),
        (Err(error), _) | (_, Err(error)) => Some(error),
    };
    match problem {
        None => {
            println!("==> They match");
            ExitCode::SUCCESS
        }
        Some(problem) => {
            eprintln!("{problem}");
            eprintln!("==> FAILED: make the same change in both Dockerfiles");
            ExitCode::FAILURE
        }
    }
}

/// What is wrong, if anything, with the shared part of the two files.
fn problem(local: &str, github: &str) -> Option<String> {
    let (Some(local), Some(github)) = (shared_part(local), shared_part(github)) else {
        return Some(format!(
            "Both Dockerfiles need the line starting \"{MARKER}\""
        ));
    };
    let first_difference = local
        .iter()
        .zip(&github)
        .position(|(a, b)| a != b)
        .or_else(|| (local.len() != github.len()).then_some(local.len().min(github.len())))?;
    Some(format!(
        "Line {} after the marker differs:\n  {LOCAL}:        {}\n  {GITHUB}: {}",
        first_difference + 1,
        line_or_end(&local, first_difference),
        line_or_end(&github, first_difference),
    ))
}

/// One of the lines, or "(the end)" if the file stops before it.
fn line_or_end<'a>(lines: &[&'a str], index: usize) -> &'a str {
    lines.get(index).copied().unwrap_or("(the end)")
}

/// The lines after the marker, without line endings or trailing blank lines.
fn shared_part(text: &str) -> Option<Vec<&str>> {
    let mut lines = text.lines().map(|line| line.trim_end_matches('\r'));
    lines.by_ref().find(|line| line.starts_with(MARKER))?;
    let mut shared: Vec<&str> = lines.collect();
    while shared.last().is_some_and(|line| line.trim().is_empty()) {
        shared.pop();
    }
    Some(shared)
}
