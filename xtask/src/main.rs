//! Project helper commands, run with `cargo xtask <task>`.
//!
//! Plain Rust instead of shell scripts so the same commands work on Windows,
//! macOS, Linux and in CI without installing anything extra.

mod api;
mod sizes;
mod target_dir;
mod webgl;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "\
Usage: cargo xtask <task>

Tasks:
  test    Check formatting, type-check and lint (warnings are errors) for
          both native and the browser (wasm), then run the tests. Run this
          before merging.
  lint    The same checks without the tests, and the file size and WebGL 2
          checks. The pre-commit hook runs this.
  sizes   Lower the entries in xtask/large-files.txt to the files' sizes
          now, removing files back under the limit.
  hooks   Turn on the pre-commit hook for this checkout (once per clone).
  package Release build of the native client and server, put together
          with the game assets in package/ in cargo's target folder
          (target/package/ unless cargo is set to build elsewhere: what
          the Build workflow uploads).
";

/// A named cargo command: what `lint` and `test` are made of.
type Check = (&'static str, &'static [&'static str]);

const LINTS: &[Check] = &[
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
    // The debug API is only in builds that ask for it, so lint those too.
    (
        "Lint (native, debug API)",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--features",
            DEBUG_API,
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
];

/// The features that build the debug API in, for every crate that has it.
const DEBUG_API: &str = "client/debug-api,server/debug-api";

/// With the debug API, so its tests run too.
const TESTS: Check = ("Tests", &["test", "--workspace", "--features", DEBUG_API]);

fn main() -> ExitCode {
    match env::args().nth(1).as_deref() {
        Some("test") => all_passed([
            run_checks(LINTS.iter().chain([&TESTS])),
            sizes::check(&workspace_root()),
            webgl::check(&workspace_root()),
            no_temporary_methods(),
        ]),
        Some("lint") => all_passed([
            run_checks(LINTS),
            sizes::check(&workspace_root()),
            webgl::check(&workspace_root()),
        ]),
        Some("sizes") => sizes::update(&workspace_root()),
        Some("hooks") => hooks(),
        Some("package") => package(),
        Some("api") => api::run(&env::args().skip(2).collect::<Vec<_>>()),
        _ => {
            eprintln!(
                "{USAGE}
{}",
                api::USAGE
            );
            ExitCode::FAILURE
        }
    }
}

/// Success only if every one succeeded.
fn all_passed(results: impl IntoIterator<Item = ExitCode>) -> ExitCode {
    let results: Vec<_> = results.into_iter().collect();
    if results.iter().all(|result| *result == ExitCode::SUCCESS) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Fails if any debug API method is still named `tmp.`: those are for trying
/// things out on a branch, and are removed or renamed before merging.
fn no_temporary_methods() -> ExitCode {
    println!("==> Temporary debug API methods");
    let found = api::temporary_methods(&workspace_root());
    if found.is_empty() {
        println!("==> None");
        return ExitCode::SUCCESS;
    }
    for place in &found {
        eprintln!("{place}");
    }
    eprintln!("==> FAILED: remove these `tmp.` methods, or rename them to keep them");
    ExitCode::FAILURE
}

/// Points git at the hooks kept in the repository (`.githooks/`). Git doesn't
/// do this by itself when cloning, so that a clone can't run code on its own.
fn hooks() -> ExitCode {
    let installed = Command::new("git")
        .args(["config", "core.hooksPath", ".githooks"])
        .current_dir(workspace_root())
        .status()
        .is_ok_and(|status| status.success());
    if installed {
        println!("==> Pre-commit hook on: each commit runs `cargo xtask lint` first");
        ExitCode::SUCCESS
    } else {
        eprintln!("==> FAILED: git config core.hooksPath .githooks");
        ExitCode::FAILURE
    }
}

/// Runs every check, even if an earlier one fails, then reports all failures.
fn run_checks<'a>(checks: impl IntoIterator<Item = &'a Check>) -> ExitCode {
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
/// `assets` folder, into `package/` in cargo's target folder (usually
/// `target/package/`): the layout players get, since release builds load
/// assets from next to the executable.
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
    let target = target_dir::target_dir(&root);
    let package = target.join("package");
    println!("==> Package: {}", package.display());
    match assemble(&root, &target.join("release"), &package) {
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

/// Empties `package`, then copies in the executables from `release` and the
/// assets.
fn assemble(root: &Path, release: &Path, package: &Path) -> io::Result<()> {
    if package.exists() {
        fs::remove_dir_all(package)?;
    }
    fs::create_dir_all(package)?;
    for name in ["hall-of-worlds", "hall-of-worlds-server"] {
        let file = format!("{name}{}", env::consts::EXE_SUFFIX);
        fs::copy(release.join(&file), package.join(&file))?;
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
