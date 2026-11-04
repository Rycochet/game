//! `cargo xtask latest`: a copy of the latest development build for the owner
//! to play, in `latest/` in cargo's target folder, so a running copy never
//! gets in the way of building (Windows locks a running executable).
//!
//! The executable has its own name, so closing development copies by name
//! (`hall-of-worlds.exe`) leaves it running, and the assets go beside it,
//! which debug builds use when they are there (the source folder may change
//! or, for a worktree, be gone).

use std::fs;
use std::io;
use std::path::Path;
use std::process::ExitCode;

use super::{cargo, copy_dir, target_dir, workspace_root};

/// The copy's name, without `.exe`.
const COPY: &str = "hall-of-worlds-latest";

/// The debug API port `play.cmd` uses (and the embedded server the next),
/// away from the default 15702, so development copies and `cargo xtask api`
/// never meet the owner's game.
const PLAY_PORT: u16 = 15712;

pub fn run() -> ExitCode {
    let build = ["build", "--package", "client", "--features", "debug-api"];
    println!("==> Build: cargo {}", build.join(" "));
    if !cargo(&build) {
        eprintln!("==> FAILED: Build");
        return ExitCode::FAILURE;
    }

    let root = workspace_root();
    let target = target_dir::target_dir(&root);
    let latest = target.join("latest");
    println!("==> Latest: {}", latest.display());
    match assemble(&root, &target.join("debug"), &latest) {
        Ok(()) => {
            println!("==> Copied: start {}", latest.join(PLAY).display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("==> FAILED: Latest: {error}");
            ExitCode::FAILURE
        }
    }
}

/// What starts the copy: `play.cmd` on Windows, `play.sh` elsewhere.
#[cfg(windows)]
const PLAY: &str = "play.cmd";
#[cfg(not(windows))]
const PLAY: &str = "play.sh";

/// Copies the executable first (it fails while the copy is running, before
/// anything else is touched), then the assets afresh, then the starter.
fn assemble(root: &Path, debug: &Path, latest: &Path) -> io::Result<()> {
    fs::create_dir_all(latest)?;
    let suffix = std::env::consts::EXE_SUFFIX;
    fs::copy(
        debug.join(format!("hall-of-worlds{suffix}")),
        latest.join(format!("{COPY}{suffix}")),
    )
    .map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("{error} (if the latest copy is running, close it and run this again)"),
        )
    })?;
    let assets = latest.join("assets");
    if assets.exists() {
        fs::remove_dir_all(&assets)?;
    }
    copy_dir(&root.join("crates/client/assets"), &assets)?;
    fs::write(latest.join(PLAY), starter())
}

/// The starter's text: the debug API on, at its own port, and the game started
/// on its own, so whatever started it can finish without closing it.
#[cfg(windows)]
fn starter() -> String {
    format!(
        "@echo off\r\n\
         rem Starts the latest development build (made by cargo xtask latest).\r\n\
         set HOW_DEBUG_API={PLAY_PORT}\r\n\
         start \"\" \"%~dp0{COPY}.exe\"\r\n"
    )
}
#[cfg(not(windows))]
fn starter() -> String {
    format!(
        "#!/bin/sh\n\
         # Starts the latest development build (made by cargo xtask latest).\n\
         cd \"$(dirname \"$0\")\" && HOW_DEBUG_API={PLAY_PORT} nohup ./{COPY} >/dev/null 2>&1 &\n"
    )
}
