//! Project helper commands, run with `cargo xtask <task>`.
//!
//! Plain Rust instead of shell scripts so the same commands work on Windows,
//! macOS, Linux and in CI without installing anything extra.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
Usage: cargo xtask <task>

Tasks:
  test    Check formatting, type-check and lint (warnings are errors) for
          both native and the browser (wasm), then run the tests. This is
          what CI runs.
  package Release build of the native client and server, put together
          with the game assets in target/package/ (what the Build
          workflow uploads).";

fn main() -> ExitCode {
    match env::args().nth(1).as_deref() {
        Some("test") => test(),
        Some("package") => package(),
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

/// Builds the release executables and copies them, with the client's
/// `assets` folder, into `target/package/`: the layout players get, since
/// release builds load assets from next to the executable.
fn package() -> ExitCode {
    let build = [
        "build",
        "--release",
        "--locked",
        "--package",
        "client",
        "--package",
        "server",
    ];
    println!("==> Build: cargo {}", build.join(" "));
    if !cargo(&build) {
        eprintln!("==> FAILED: Build");
        return ExitCode::FAILURE;
    }

    let root = workspace_root();
    let package = root.join("target/package");
    println!("==> Package: {}", package.display());
    match assemble(&root, &package) {
        Ok(()) => {
            println!("==> Packaged");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("==> FAILED: Package: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Empties `package`, then copies in the executables and the assets.
fn assemble(root: &Path, package: &Path) -> io::Result<()> {
    if package.exists() {
        fs::remove_dir_all(package)?;
    }
    fs::create_dir_all(package)?;
    for name in ["hall-of-worlds", "hall-of-worlds-server"] {
        let file = format!("{name}{}", env::consts::EXE_SUFFIX);
        fs::copy(root.join("target/release").join(&file), package.join(&file))?;
    }
    copy_dir(&root.join("crates/client/assets"), &package.join("assets"))
}

/// Copies a folder and everything in it.
fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// The workspace folder (this crate's parent).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is inside the workspace")
        .to_path_buf()
}

/// Runs cargo (the same one that launched us) and returns whether it succeeded.
fn cargo(args: &[&str]) -> bool {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    Command::new(cargo)
        .args(args)
        .status()
        .is_ok_and(|status| status.success())
}
