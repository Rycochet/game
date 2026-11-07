//! `cargo xtask scenario <file> [--start]`: a list of debug API calls kept in a
//! file and played in order, to set a test up again after any change (a Dev
//! Area with some wolves, an item given, a screenshot). Each line is what would
//! follow `cargo xtask api`; `#` starts a comment. With `--start` it builds the
//! client with the debug API, starts it on its own scratch data folder, plays the
//! file and closes it again.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode};
use std::time::{Duration, Instant};
use std::{env, fs, thread};

use crate::target_dir::target_dir;

/// How long a started game may take to answer.
const START_WAIT: Duration = Duration::from_secs(120);

pub const USAGE: &str = "\
  scenario  Play a file of debug API calls, one `cargo xtask api` line each:
              cargo xtask scenario <file> [--start]
            `#` starts a comment. --start builds and starts the client (debug
            API on, scratch data, title \"Dev:\"), plays the file, then closes it.
            Without it the game must already be running. See scenarios/.";

pub fn run(root: &Path, args: &[String]) -> ExitCode {
    let start = args.iter().any(|arg| arg == "--start");
    let Some(file) = args.iter().find(|arg| !arg.starts_with("--")) else {
        eprintln!("Which file?\n\nUsage:\n{USAGE}");
        return ExitCode::FAILURE;
    };
    let text = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("==> Can't read {file}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut game = None;
    if start {
        match start_game(root) {
            Ok(child) => game = Some(child),
            Err(error) => {
                eprintln!("==> {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    let played = play(&text);
    if let Some(mut child) = game {
        let _ = child.kill();
        let _ = child.wait();
    }
    match played {
        Ok(count) => {
            println!("==> Played {count} calls from {file}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

/// Plays each call in turn, stopping at the first that fails. How many.
fn play(text: &str) -> Result<usize, String> {
    let calls: Vec<(usize, Vec<String>)> = text
        .lines()
        .enumerate()
        .map(|(at, line)| (at + 1, words(line)))
        .filter(|(_, words)| !words.is_empty())
        .collect();
    for (line, args) in &calls {
        println!("==> Line {line}: {}", args.join(" "));
        crate::api::perform(args).map_err(|error| format!("{error}\n==> FAILED at line {line}"))?;
    }
    Ok(calls.len())
}

/// A line's words, as a shell would split them: by spaces, keeping `"..."`
/// together (without the quotes), the rest of the line from a `{` or `[` as one
/// (JSON params), and nothing from a `#` outside quotes.
pub(crate) fn words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    let mut chars = line.trim().chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => quoted = !quoted,
            '#' if !quoted && word.is_empty() => break,
            '{' | '[' if !quoted && word.is_empty() => {
                word.push(c);
                word.extend(chars.by_ref());
                break;
            }
            c if c.is_whitespace() && !quoted => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            c => word.push(c),
        }
    }
    let word = word.trim().to_owned();
    if !word.is_empty() {
        words.push(word);
    }
    words
}

/// Builds the client with the debug API and starts it, as a development copy
/// with its own data, waiting until its debug API answers.
fn start_game(root: &Path) -> Result<Child, String> {
    println!("==> Build: cargo build -p client --features debug-api");
    let built = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["build", "-p", "client", "--features", "debug-api"])
        .current_dir(root)
        .status()
        .map_err(|error| format!("Can't run cargo: {error}"))?;
    if !built.success() {
        return Err("FAILED: Build".into());
    }
    let exe = if cfg!(windows) {
        "hall-of-worlds.exe"
    } else {
        "hall-of-worlds"
    };
    let program = target_dir(root).join("debug").join(exe);
    let data: PathBuf = env::temp_dir().join(format!("how-scenario-{}", std::process::id()));
    println!(
        "==> Start: {} (data in {})",
        program.display(),
        data.display()
    );
    let mut child = Command::new(&program)
        .env("HOW_DEBUG_API", "true")
        .env("HOW_DEBUG_TITLE", "Dev:")
        .env("HOW_DATA", &data)
        .env("HOW_DEBUG_SKIP_MENU", "true")
        .current_dir(root)
        .spawn()
        .map_err(|error| format!("Can't start {}: {error}", program.display()))?;
    let began = Instant::now();
    while !crate::api::answers() {
        if began.elapsed() > START_WAIT {
            let _ = child.kill();
            return Err("The game's debug API never answered".into());
        }
        thread::sleep(Duration::from_millis(500));
    }
    Ok(child)
}

#[cfg(test)]
mod tests;
