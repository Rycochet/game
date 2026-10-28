//! Project helper commands, run with `cargo xtask <task>`.
//!
//! Plain Rust instead of shell scripts so the same commands work on Windows,
//! macOS, Linux and in CI without installing anything extra.

use std::env;
use std::process::{Command, ExitCode};

const USAGE: &str = "\
Usage: cargo xtask <task>

Tasks:
  test    Check formatting, type-check and lint (warnings are errors) for
          both native and the browser (wasm), then run the tests. This is
          what CI runs.";

fn main() -> ExitCode {
    match env::args().nth(1).as_deref() {
        Some("test") => test(),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

/// Runs every check, even if an earlier one fails, then reports all failures.
fn test() -> ExitCode {
    let checks: &[(&str, &[&str])] = &[
        ("Formatting", &["fmt", "--all", "--check"]),
        (
            "Lint (native)",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        ),
        // Only the client (and the shared crate it uses) runs in the browser.
        (
            "Lint (web)",
            &[
                "clippy",
                "--package",
                "client",
                "--target",
                "wasm32-unknown-unknown",
                "--",
                "-D",
                "warnings",
            ],
        ),
        ("Tests", &["test", "--workspace"]),
    ];

    let mut failed = Vec::new();
    for (name, args) in checks {
        println!("==> {name}: cargo {}", args.join(" "));
        if !cargo(args) {
            failed.push(*name);
        }
    }

    if failed.is_empty() {
        println!("==> All checks passed");
        ExitCode::SUCCESS
    } else {
        eprintln!("==> FAILED: {}", failed.join(", "));
        ExitCode::FAILURE
    }
}

/// Runs cargo (the same one that launched us) and returns whether it succeeded.
fn cargo(args: &[&str]) -> bool {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    Command::new(cargo)
        .args(args)
        .status()
        .is_ok_and(|status| status.success())
}
